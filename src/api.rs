use crate::{db, error, opt};
use serde::Serialize;

use tide::log::LogMiddleware;
use tide::utils::After;
use tide::Body;
use tide::Request;
use url::Url;

// FIXME: id vs name ??
#[derive(Debug, Serialize)]
struct ReadResponse {
    service_name: String,
    endpoint_name: String,
    instance_name: String,
    endpoint_url: Url,
}

impl ReadResponse {
    fn from_endpoint_url(endpoint: db::EndpointKey, url: Url) -> Self {
        ReadResponse {
            service_name: endpoint.service_name,
            instance_name: endpoint.instance_name,
            endpoint_name: endpoint.endpoint_name,
            endpoint_url: url,
        }
    }
}

fn endpoint_key_from_req<S>(req: &Request<S>) -> tide::Result<db::EndpointKey> {
    Ok(db::EndpointKey {
        service_name: req.param("service_name")?.into(),
        instance_name: req.param("instance_name")?.into(),
        endpoint_name: req.param("endpoint_name")?.into(),
    })
}

#[derive(Clone)]
struct State {
    opt: opt::Opt,
    db: db::Db,
}

pub async fn run(opt: opt::Opt, db: db::Db) -> error::Result<()> {
    log::info!("Starting API");
    let listen = opt.listen.clone();
    let state = State { db, opt };
    let mut app = tide::with_state(state);

    app.with(LogMiddleware::new());
    // FIXME: error handling
    app.with(After(|mut res: tide::Response| async {
        if let Some(err) = res.error() {
            log::error!("Request failed: {:?}", &err);
            let msg = format!("Error: {:?}", &err);
            res.set_body(msg);
        }
        Ok(res)
    }));

    app.at("/").get(root);
    app.at("/ping").get(ping);
    app.at("/openapi.yaml").get(openapi);
    app.at("/all").get(read_all);
    app.at("/services/:service_name/instances/:instance_name/endpoints/:endpoint_name")
        .post(register)
        .get(read);
    app.at("/services/:service_name/instances/:instance_name/endpoints/:endpoint_name/stats")
        .get(read_stats);

    log::info!("API will listen on {}", &listen);
    app.listen(listen)
        .await
        .map_err(error::Error::RunApiError)?;
    Ok(())
}

async fn root(_req: Request<State>) -> tide::Result {
    let s = format!("Welcome to eccer v{}", env!("CARGO_PKG_VERSION"));
    Ok(s.into())
}

async fn ping(_req: Request<State>) -> tide::Result {
    Ok("ok".into())
}

async fn openapi(_req: Request<State>) -> tide::Result {
    let mut body = Body::from_string(include_str!("../openapi.yaml").into());
    body.set_mime("text/yaml");
    Ok(body.into())
}

async fn register(mut req: Request<State>) -> tide::Result {
    let endpoint = endpoint_key_from_req(&req)?;
    let body = req.body_string().await?;
    // FIXME: json api
    let endpoint_url = Url::parse(&body)?;
    let mut db = req.state().db.clone();
    db.add_endpoint_url(endpoint.clone(), &endpoint_url).await?;
    let read_response = ReadResponse::from_endpoint_url(endpoint, endpoint_url);
    // FIXME:  status code
    Ok(Body::from_json(&read_response)?.into())
}

async fn read(req: Request<State>) -> tide::Result {
    let endpoint = endpoint_key_from_req(&req)?;
    let mut db = req.state().db.clone();
    let endpoint_url = db.get_endpoint_url(endpoint.clone()).await?;
    let read_response = ReadResponse::from_endpoint_url(endpoint, endpoint_url);
    Ok(Body::from_json(&read_response)?.into())
}

async fn read_all(req: Request<State>) -> tide::Result {
    let mut db = req.state().db.clone();
    let endpoint_urls = db.get_all_endpoint_urls().await?;
    Ok(Body::from_json(&endpoint_urls)?.into())
}

async fn read_stats(req: Request<State>) -> tide::Result {
    let endpoint = endpoint_key_from_req(&req)?;
    let mut db = req.state().db.clone();
    let endpoint_stats = db.get_endpoint_stats(endpoint.clone()).await?;
    Ok(Body::from_json(&endpoint_stats)?.into())
}
