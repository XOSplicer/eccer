#[derive(Clone, Debug, clap::Parser)]
#[command(version, about)]
pub struct Opt {
    #[command(subcommand)]
    pub command: Option<Command>,
    /// Address for the API to listen on
    // TODO: move to server subcommand
    #[arg(short, long, env, default_value = "localhost:4447")]
    pub listen: String,
    /// Address for the metrics endpoint to listen on
    #[arg(short, long, env, default_value = "localhost:4490")]
    pub metrics: String,
    /// List of etcd endpoints for database connection
    #[arg(short, long, env, default_value = "localhost:2379")]
    pub etcd_endpoints: Vec<String>,
    /// Username for etcd
    #[arg(long, env)]
    pub etcd_user: Option<String>,
    /// Password for etcd
    #[arg(long, env, hide_env_values = true)]
    pub etcd_password: Option<String>,
    /// Key prefix to use in etcd
    #[arg(long, env, default_value = "eccer")]
    pub etcd_prefix: String,
    /// Address for the NATS message queue
    #[arg(short, long, env, default_value = "localhost:4222")]
    pub nats_address: String,
    #[arg(long, env)]
    /// Username for the NATS message queue
    pub nats_user: Option<String>,
    /// Password for the NATS message queue
    #[arg(long, env, hide_env_values = true)]
    pub nats_password: Option<String>,
    /// Queue prefix to use in NATS
    #[arg(long, env, default_value = "eccer")]
    pub nats_prefix: String,
    /// [seconds] How often to dispatch ping commands to workers to check for live services
    #[arg(short, long, env, default_value = "60")]
    pub dispatch_interval: u64,
    #[arg(long, env, default_value = "1")]
    /// [seconds] How long to wait for a ping request before timeout
    pub request_timeout: u64,
    /// After how many failed ping requests should a service be deleted from the registry
    #[arg(long, env)]
    pub delete_after_failures: Option<u64>,
}

#[derive(Clone, Debug, clap::Subcommand)]
pub enum Command {
    /// Run API server process
    Server,
    /// Run worker process
    Worker,
}

impl Default for Command {
    fn default() -> Self {
        Command::Server
    }
}

impl Opt {
    pub fn etcd_connect_options(&self) -> etcd_client::ConnectOptions {
        match (self.etcd_user.as_ref(), self.etcd_password.as_ref()) {
            (None, None) => etcd_client::ConnectOptions::new(),
            (Some(user), Some(password)) => {
                etcd_client::ConnectOptions::new().with_user(user, password)
            }
            (Some(user), None) => etcd_client::ConnectOptions::new().with_user(user, ""),
            (None, Some(password)) => etcd_client::ConnectOptions::new().with_user("", password),
        }
    }

    pub fn nats_connect_options(&self) -> nats::asynk::Options {
        match (self.nats_user.as_ref(), self.nats_password.as_ref()) {
            (None, None) => nats::asynk::Options::new(),
            (Some(user), Some(password)) => nats::asynk::Options::with_user_pass(user, password),
            (Some(user), None) => nats::asynk::Options::with_user_pass(user, ""),
            (None, Some(password)) => nats::asynk::Options::with_user_pass("", password),
        }
        .with_name("eccer")
    }
}
