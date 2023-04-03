use env_logger::Env;
use futures_util::StreamExt;
use std::convert::TryInto;
use std::time::{Duration, Instant};
use structopt::StructOpt;
use surf::{Client, Config};
use tokio;

use crate::{db, dispatch, opt, queue};

pub async fn run_worker(opt: opt::Opt, mut db: db::Db, queue: queue::Queue) -> anyhow::Result<()> {
    log::info!("Running eccer worker");
    let input = queue.subscribe_ping().await?;
    futures_util::pin_mut!(input);
    let client: Client = Config::new()
        .set_timeout(Some(Duration::from_secs(opt.request_timeout)))
        .try_into()?;
    while let Some(key) = input.next().await {
        let key = key?;
        let url = db.get_endpoint_url(key.clone()).await?;
        log::info!("GET ping key={} url={}", &key, &url);
        let start = Instant::now();
        let res_status = client.get(&url).await.map(|res| res.status());
        let is_success = match res_status {
            Err(_) => false,
            Ok(status) => status.is_success(),
        };
        let duration = Instant::now().duration_since(start);
        let duration_ms = duration.as_micros() as f64 / 1000.0;
        log::info!(
            "GET repsonse key={} url={} response={:?} success={} duration={:.2}ms",
            &key,
            &url,
            res_status,
            is_success,
            duration_ms
        );
        if is_success {
            db.record_endpoint_success(key, chrono::offset::Utc::now())
                .await?;
        } else {
            db.record_endpoint_failure(key, chrono::offset::Utc::now())
                .await?;
        }
    }
    Ok(())
}
