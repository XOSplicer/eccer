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
}