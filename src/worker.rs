use env_logger::Env;
use futures_util::StreamExt;
use structopt::StructOpt;
use tokio;

use crate::{db, dispatch, opt, queue};

pub async fn run_worker(_opt: opt::Opt, mut db: db::Db, queue: queue::Queue) -> anyhow::Result<()> {
    log::info!("Running eccer worker");
    let input = queue.subscribe_ping().await?;
    futures_util::pin_mut!(input);
    while let Some(key) = input.next().await {
        let key = key?;
        log::info!("got {}", &key);
        let url = db.get_endpoint_url(key).await?;
        log::info!("url {}", &url);
        let res_status = surf::get(&url).await.map(|res| res.status());
        log::info!("res_status {:?}", &res_status);
    }
    Ok(())
}
