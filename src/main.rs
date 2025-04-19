#![forbid(unsafe_code)]
#![allow(dead_code)]

// TODO: replace tide with tokio compatible http server instead of async-std

use clap::Parser;
use tokio;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod api;
mod db;
mod dispatch;
mod error;
mod metric_names;
mod metrics_endpoint;
mod opt;
mod queue;
mod server;
mod worker;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let opt: opt::Opt = opt::Opt::parse();

    info!(
        "Connecting to etcd, using endpoints {:?}",
        &opt.etcd_endpoints
    );
    let etcd_options = opt.etcd_connect_options();
    let etcd = etcd_client::Client::connect(&opt.etcd_endpoints, Some(etcd_options)).await?;
    let db = db::Db::new(etcd, opt.etcd_prefix.clone());
    info!("Established connection to etcd");

    info!("Connecting to nats, using address {}", &opt.nats_address);
    let nats_options = opt.nats_connect_options();
    let nats = nats_options.connect(&opt.nats_address).await?;
    let queue = queue::Queue::new(nats, opt.nats_prefix.clone());
    info!("Established connection to nats");

    match opt.command.clone().unwrap_or_default() {
        opt::Command::Server => server::run_server(opt, db, queue).await?,
        opt::Command::Worker => worker::run_worker(opt, db, queue).await?,
    }

    Ok(())
}
