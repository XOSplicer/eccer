use tokio;
use tracing::info;

use crate::{api, db, dispatch, opt, queue};

pub async fn run_server(opt: opt::Opt, db: db::Db, queue: queue::Queue) -> anyhow::Result<()> {
    info!("Running eccer server");
    tokio::try_join!(
        api::run(opt.clone(), db.clone()),
        dispatch::run(opt.clone(), db, queue),
    )?;

    Ok(())
}
