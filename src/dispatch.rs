use crate::{db, error, opt, queue};
use std::time::Duration;
use tokio;

pub async fn run(opt: opt::Opt, mut db: db::Db, queue: queue::Queue) -> error::Result<()> {
    log::info!("Starting dispatch");
    loop {
        tokio::time::sleep(Duration::from_secs(opt.dispatch_interval)).await;
        log::info!("Dispatching messages");
        queue
            .publish_hello_world()
            .await
            .map_err(error::Error::RunDispatchError)?;
        for entry in db.get_all_endpoint_urls().await? {
            queue
                .publish_ping(entry.key)
                .await
                .map_err(error::Error::RunDispatchError)?;
        }
    }
    Ok(())
}
