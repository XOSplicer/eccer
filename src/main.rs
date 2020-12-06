use structopt::StructOpt;
use tide::prelude::*;
use tide::utils::After;
use tide::Request;
use url::Url;

mod db;
mod error;
mod opt;

// TODO: restructure

// FIXME: id vs name ??
#[derive(Debug, Serialize)]
struct ReadResponse {
    service_name: String,
    endpoint_name: String,
    instance_name: String,
    endpoint_url: Url,
}

#[derive(Clone)]
struct State {
    opt: opt::Opt,
    db: db::Db,
}

#[tokio::main]
async fn main() -> tide::Result<()> {
    env_logger::init();
    let opt: opt::Opt = opt::Opt::from_args();
    let etcd_options = opt.etcd_connect_options();
    let etcd = etcd_client::Client::connect(&opt.etcd_endpoints, Some(etcd_options)).await?;
    let db = db::Db::new(opt.etcd_prefix.clone(), etcd);
    let state = State {
        db,
        opt: opt.clone(),
    };
    let mut app = tide::with_state(state);
    // FIXME: error handling
    app.with(After(|mut res: tide::Response| async {
        if let Some(err) = res.error() {
            log::error!("request failed: {:?}", &err);
            let msg = format!("Error: {:?}", &err);
            res.set_body(msg);
        }
        Ok(res)
    }));
    app.at("/services/:service_name/instances/:instance_name/endpoints/:endpoint_name")
        .post(register)
        .get(read);
    app.listen(opt.listen).await?;

    Ok(())
}

async fn register(mut req: Request<State>) -> tide::Result {
    let service_name: String = req.param("service_name")?.into();
    let instance_name: String = req.param("instance_name")?.into();
    let endpoint_name: String = req.param("endpoint_name")?.into();
    let endpoint = db::EndpointKey {
        service_name: service_name.clone(),
        instance_name: instance_name.clone(),
        endpoint_name: endpoint_name.clone(),
    };
    let body = req.body_string().await?;
    // FIXME: json api
    let endpoint_url = Url::parse(&body)?;
    let mut db = req.state().db.clone();
    db.add_endpoint_url(endpoint, &endpoint_url).await?;
    let read_response = ReadResponse {
        service_name,
        instance_name,
        endpoint_name,
        endpoint_url,
    };
    // FIXME:  status code
    Ok(tide::Body::from_json(&read_response)?.into())
}

async fn read(req: Request<State>) -> tide::Result {
    let service_name: String = req.param("service_name")?.into();
    let instance_name: String = req.param("instance_name")?.into();
    let endpoint_name: String = req.param("endpoint_name")?.into();
    let endpoint = db::EndpointKey {
        service_name: service_name.clone(),
        instance_name: instance_name.clone(),
        endpoint_name: endpoint_name.clone(),
    };
    let mut db = req.state().db.clone();
    let endpoint_url = db.get_endpoint_url(endpoint).await?;
    let read_response = ReadResponse {
        service_name,
        instance_name,
        endpoint_name,
        endpoint_url,
    };
    Ok(tide::Body::from_json(&read_response)?.into())
}
