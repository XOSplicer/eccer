use env_logger::Env;
use futures_util::StreamExt;
use structopt::StructOpt;
use tokio;

use crate::{db, dispatch, opt, queue};

pub async fn run_worker(_opt: opt::Opt, _db: db::Db, queue: queue::Queue) -> anyhow::Result<()> {
    log::info!("Running eccer worker");
    let input = queue.subscribe_ping().await?;
    futures_util::pin_mut!(input);
    while let Some(key) = input.next().await {
        log::info!("got {}", key?);
    }
    Ok(())
}
