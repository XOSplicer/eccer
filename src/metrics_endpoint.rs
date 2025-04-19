use crate::opt;
use axum::{response::Html, routing::get, Router};
use axum_prometheus::PrometheusMetricLayer;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

pub async fn run(opt: opt::Opt) -> anyhow::Result<()> {
    info!("Starting metrics endpoint");
    let listen = opt.metrics.clone();
    // TODO: add other metrics to prometheus if possible
    // eg. from nats connection, etcd connection
    // and add our own metrics where possible eg pending tasks, num workers etc
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
    let app = Router::new()
        .route("/", get(root))
        .route("/ping", get(ping))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(prometheus_layer)
        .layer(TraceLayer::new_for_http());
    info!("Metrics endpoint will listen on http://{}", &listen);
    let listener = TcpListener::bind(&listen).await.unwrap();
    axum::serve(listener, app)
        .await
        .expect("Error while running metrics endpoint");
    Ok(())
}

async fn root() -> Html<&'static str> {
    Html(
        r#"
<html>
    <body>
        <a href="/ping">/ping</a><br/>
        <a href="/metrics">/metrics</a>
    </body>
</html>
"#,
    )
}

async fn ping() -> &'static str {
    "ok"
}
