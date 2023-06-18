use futures_util::StreamExt;
use reqwest::Client;
use std::time::{Duration, Instant};
use tracing::info;

use crate::{db, opt, queue};

pub async fn run_worker(opt: opt::Opt, mut db: db::Db, queue: queue::Queue) -> anyhow::Result<()> {
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
        let start = Instant::now();
        let res_status = client.get(url.clone()).send().await.map(|res| res.status());
        let is_success = match res_status {
            Err(_) => false,
            Ok(status) => status.is_success(),
        };
        let duration = Instant::now().duration_since(start);
        let duration_ms = duration.as_micros() as f64 / 1000.0;
        info!(
            "GET repsonse key={} url={} response={:?} success={} duration={:.2}ms",
            &key, &url, res_status, is_success, duration_ms
        );
        if is_success {
            db.record_endpoint_success(key.clone(), chrono::offset::Utc::now())
                .await?;
        } else {
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
