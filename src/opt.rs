#[derive(Clone, Debug, structopt::StructOpt)]
pub struct Opt {
    #[structopt(short, long, env, default_value = "localhost:4447")]
    pub listen: String,
    #[structopt(short, long, env, default_value = "localhost:2379")]
    pub etcd_endpoints: Vec<String>,
    #[structopt(long, env)]
    pub etcd_user: Option<String>,
    #[structopt(long, env, hide_env_values = true)]
    pub etcd_password: Option<String>,
    #[structopt(long, env, default_value = "eccer")]
    pub etcd_prefix: String,
    #[structopt(short, long, env, default_value = "localhost:4222")]
    pub nats_address: String,
    #[structopt(long, env)]
    pub nats_user: Option<String>,
    #[structopt(long, env, hide_env_values = true)]
    pub nats_password: Option<String>,
    #[structopt(long, env, default_value = "eccer")]
    pub nats_prefix: String,
    #[structopt(short, long, env, default_value = "60")]
    pub dispatch_interval: u64,
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
