#![forbid(unsafe_code)]
#![allow(dead_code)]

// TODO: replace tide with tokio compatible http server instead of async-std
use tide::prelude::*;
use tokio;
use structopt::StructOpt;
use env_logger::Env;
use futures_util::StreamExt;
use futures_util::TryStreamExt;

mod db;
mod error;
mod opt;
mod api;
mod dispatch;
mod queue;

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let opt: opt::Opt = opt::Opt::from_args();

    log::info!("Connecting to etcd, using endpoints {:?}", &opt.etcd_endpoints);
    let etcd_options = opt.etcd_connect_options();
    let etcd = etcd_client::Client::connect(&opt.etcd_endpoints, Some(etcd_options)).await?;
    let db = db::Db::new(etcd, opt.etcd_prefix.clone());
    log::info!("Established connection to etcd");

    log::info!("Connecting to nats, using address {}", &opt.nats_address);
    let nats_options = opt.nats_connect_options();
    let nats = nats_options.connect(&opt.nats_address).await?;
    let queue = queue::Queue::new(nats, opt.nats_prefix.clone());
    log::info!("Established connection to nats");

    match opt.command.clone().unwrap_or_default() {
        opt::Command::Server => run_server(opt, db, queue).await?,
        opt::Command::Worker => run_worker(opt, db, queue).await?,
    }

    Ok(())
}

async fn run_server(opt: opt::Opt, db: db::Db, queue: queue::Queue) -> anyhow::Result<()> {
    log::info!("Running eccer server");
    tokio::try_join!(
        api::run(opt.clone(), db.clone()),
        dispatch::run(opt.clone(), db, queue),
    )?;

    Ok(())
}

async fn run_worker(opt: opt::Opt, db: db::Db, queue: queue::Queue) -> anyhow::Result<()> {
    log::info!("Running eccer worker");
    let input = queue.subscribe_ping().await?;
    futures_util::pin_mut!(input);
    while let Some(key) = input.next().await {
        log::info!("got {}", key?);
    }
    Ok(())
}

