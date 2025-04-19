use crate::{
    db::{self, EndpointRecord, EndpointStats},
    opt,
};
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use axum_prometheus::PrometheusMetricLayer;
use serde::Serialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use url::Url;

#[derive(Debug, Serialize)]
struct ReadResponse {
    key: db::EndpointKey,
    endpoint_url: Url,
}

impl ReadResponse {
    fn from_endpoint_url(endpoint: db::EndpointKey, url: Url) -> Self {
        ReadResponse {
            key: endpoint,
            endpoint_url: url,
        }
    }
}

fn endpoint_key_from_path(
    Path((service_name, instance_name, endpoint_name)): Path<(String, String, String)>,
) -> db::EndpointKey {
    db::EndpointKey {
        service_name: service_name,
        instance_name: instance_name,
        endpoint_name: endpoint_name,
    }
}

#[derive(Clone)]
struct AppState {
    opt: opt::Opt,
    db: db::Db,
}

pub async fn run(opt: opt::Opt, db: db::Db) -> anyhow::Result<()> {
    info!("Starting API");
    let listen = opt.listen.clone();
    let shared_state = Arc::new(AppState { db, opt });

    let prometheus_layer = PrometheusMetricLayer::new();
    let app = Router::new()
        .route("/", get(root))
        .route("/ping", get(ping))
        .route("/openapi.yaml", get(openapi))
        .route("/all", get(read_all))
        .route(
            "/services/{service_name}/instances/{instance_name}/endpoints/{endpoint_name}",
            get(read).post(register),
        )
        .route(
            "/key/{service_name}/{instance_name}/{endpoint_name}",
            get(read).post(register),
        )
        .route(
            "/services/{service_name}/instances/{instance_name}/endpoints/{endpoint_name}/stats",
            get(read_stats),
        )
        .route(
            "/key/{service_name}/{instance_name}/{endpoint_name}/stats",
            get(read_stats),
        )
        .with_state(shared_state)
        .layer(prometheus_layer)
        .layer(TraceLayer::new_for_http());
    info!("API will listen on http://{}", &listen);
    let listener = TcpListener::bind(&listen).await.unwrap();
    axum::serve(listener, app)
        .await
        .expect("Error while running API");
    Ok(())
}

async fn root() -> String {
    // TODO use Html<String>
    // TODO: print more info, like nats
    // eg INFO {"server_id":"NDXNA2LRDBHRQOSXJYFZC7X3UNJAAFEY5OY4GDZKLNPKOAQ7QGRJMOWX","server_name":"NDXNA2LRDBHRQOSXJYFZC7X3UNJAAFEY5OY4GDZKLNPKOAQ7QGRJMOWX","version":"2.9.15","proto":1,"git_commit":"b91fa85","go":"go1.19.6","host":"0.0.0.0","port":4222,"headers":true,"max_payload":1048576,"client_id":12,"client_ip":"172.19.0.1","cluster":"my_cluster","connect_urls":["172.19.0.4:4222","172.19.0.6:4222","172.19.0.7:4222"]}
    // -ERR 'Unknown Protocol Operation'
    format!("Welcome to eccer v{}", env!("CARGO_PKG_VERSION"))
}

async fn ping() -> &'static str {
    "ok"
}

async fn openapi() -> impl IntoResponse {
    // TODO: generate actual description
    (
        [(header::CONTENT_TYPE, "text/yaml")],
        include_str!("../openapi.yaml"),
    )
}

type EndpointKeyPath = Path<(String, String, String)>;

// TODO: research zookeper / consul /  other apis to model after or be compatible with

// FIXME: turn this into a json and raw api
async fn register(
    State(state): State<Arc<AppState>>,
    path: EndpointKeyPath,
    body: String,
) -> Result<(StatusCode, Json<ReadResponse>), StatusCode> {
    let endpoint = endpoint_key_from_path(path);
    let endpoint_url = Url::parse(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let mut db = state.db.clone();
    db.add_endpoint_url(endpoint.clone(), &endpoint_url)
        .await
        .map_err(|err| {
            error!(error = %err, "Could not add endpoint url");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let read_response = ReadResponse::from_endpoint_url(endpoint, endpoint_url);
    Ok((StatusCode::CREATED, Json(read_response)))
}

async fn read(
    State(state): State<Arc<AppState>>,
    path: EndpointKeyPath,
) -> Result<Json<ReadResponse>, StatusCode> {
    let endpoint = endpoint_key_from_path(path);
    let mut db = state.db.clone();
    let endpoint_url = db
        .get_endpoint_url(endpoint.clone())
        .await
        .map_err(|err| match err {
            crate::error::Error::NotFound => StatusCode::NOT_FOUND,
            _ => {
                error!(error = %err, "Could get endpoint url");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    let read_response = ReadResponse::from_endpoint_url(endpoint, endpoint_url);
    Ok(Json(read_response))
}

async fn read_all(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<EndpointRecord>>, StatusCode> {
    let mut db = state.db.clone();
    let endpoint_urls = db.get_all_endpoint_urls().await.map_err(|err| {
        error!(error = %err, "Could not read all endpoint urls");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(endpoint_urls))
}

async fn read_stats(
    State(state): State<Arc<AppState>>,
    path: EndpointKeyPath,
) -> Result<Json<EndpointStats>, StatusCode> {
    let endpoint = endpoint_key_from_path(path);
    let mut db = state.db.clone();
    let endpoint_stats =
        db.get_endpoint_stats(endpoint.clone())
            .await
            .map_err(|err| match err {
                crate::error::Error::NotFound => StatusCode::NOT_FOUND,
                _ => {
                    error!(error = %err, "Could not get endpoint stats");
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            })?;
    Ok(Json(endpoint_stats))
}
