// TODO: replace tide with tokio compatible http server instead of async-std
use tide::prelude::*;
use tokio;
use structopt::StructOpt;
use env_logger::Env;

mod db;
mod error;
mod opt;
mod api;
mod dispatch;
mod queue;

#[tokio::main]
async fn main() -> tide::Result<()> {

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let opt: opt::Opt = opt::Opt::from_args();
    let etcd_options = opt.etcd_connect_options();

    log::info!("Connecting to etcd, using endpoints {:?}", &opt.etcd_endpoints);
    let etcd = etcd_client::Client::connect(&opt.etcd_endpoints, Some(etcd_options)).await?;
    let db = db::Db::new(opt.etcd_prefix.clone(), etcd);
    log::info!("Established connection to etcd");

    log::info!("Starting API");
    api::run(opt, db).await?;

    Ok(())
}

