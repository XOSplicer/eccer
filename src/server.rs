use env_logger::Env;
use futures_util::StreamExt;
use structopt::StructOpt;
use tokio;

use crate::{api, db, dispatch, opt, queue};

pub async fn run_server(opt: opt::Opt, db: db::Db, queue: queue::Queue) -> anyhow::Result<()> {
    log::info!("Running eccer server");
    tokio::try_join!(
        api::run(opt.clone(), db.clone()),
        dispatch::run(opt.clone(), db, queue),
    )?;

    Ok(())
}
