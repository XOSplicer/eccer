use futures_util::StreamExt;
use metrics::{counter, histogram};
use reqwest::Client;
use std::time::{Duration, Instant};
use tracing::info;

use crate::{
    db,
    metric_names::{
        WORKER_REQUESTS_DURATION_SECONDS, WORKER_REQUESTS_FAILURE_TOTAL,
        WORKER_REQUESTS_FINISHED_TOTAL, WORKER_REQUESTS_SUCCESS_TOTAL, WORKER_REQUESTS_TOTAL,
    },
    metrics_endpoint, opt, queue,
};

pub async fn run_worker(opt: opt::Opt, db: db::Db, queue: queue::Queue) -> anyhow::Result<()> {
    info!("Running eccer server");
    tokio::try_join!(
        run_worker_task(opt.clone(), db, queue),
        metrics_endpoint::run(opt.clone()),
    )?;
    Ok(())
}

pub async fn run_worker_task(
    opt: opt::Opt,
    mut db: db::Db,
    queue: queue::Queue,
) -> anyhow::Result<()> {
    info!("Running eccer worker");
    let input = queue.subscribe_ping().await?;
    futures_util::pin_mut!(input);
    let client = Client::builder()
        .timeout(Duration::from_secs(opt.request_timeout))
        .build()?;
    while let Some(key) = input.next().await {
        let key = key?;
        let url = db.get_endpoint_url(key.clone()).await?;
        info!("GET ping key={} url={}", &key, &url);
        counter!(WORKER_REQUESTS_TOTAL).increment(1);
        let start = Instant::now();
        let res_status = client.get(url.clone()).send().await.map(|res| res.status());
        let is_success = match res_status {
            Err(_) => false,
            Ok(status) => status.is_success(),
        };
        let duration = Instant::now().duration_since(start);
        let duration_ms = duration.as_micros() as f64 / 1000.0;
        // TODO: metrics: count total, count success, count failure, (do not include label of key, url), histogram duration
        info!(
            "GET repsonse key={} url={} response={:?} success={} duration={:.2}ms",
            &key, &url, res_status, is_success, duration_ms
        );
        counter!(WORKER_REQUESTS_FINISHED_TOTAL).increment(1);
        histogram!(WORKER_REQUESTS_DURATION_SECONDS).record(duration_ms / 1000.0);
        if is_success {
            counter!(WORKER_REQUESTS_SUCCESS_TOTAL).increment(1);
            db.record_endpoint_success(key.clone(), chrono::offset::Utc::now())
                .await?;
        } else {
            counter!(WORKER_REQUESTS_FAILURE_TOTAL).increment(1);
            let failures = db
                .record_endpoint_failure(key.clone(), chrono::offset::Utc::now())
                .await?;
            if let Some(max_failures) = opt.delete_after_failures {
                if failures >= max_failures {
                    info!(
                        "DELETE key={} failures={} max_failures={}",
                        &key, failures, max_failures,
                    );
                    db.delete_endpoint(key.clone()).await?;
                }
            }
        }
    }
    Ok(())
}
