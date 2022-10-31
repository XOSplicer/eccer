use crate::{opt, db, error, queue};
use std::time::Duration;
use tokio;


pub async fn run(opt: opt::Opt, db: db::Db, queue: queue::Queue) -> error::Result<()> {
    log::info!("Starting dispatch");
    loop {
        tokio::time::sleep(Duration::from_secs(opt.dispatch_interval)).await;
        log::info!("Dispatching messages");
        queue.publish_hello_world().await.map_err(error::Error::RunDispatchError)?;
    }
    Ok(())
}