
pub struct Queue {
    client: nats::asynk::Connection,
    prefix: String,
}

impl Queue {
    pub fn new(client: nats::asynk::Connection, prefix: String) -> Self {
        Queue { client, prefix }
    }

    pub async fn publish_hello_world(&self) -> std::io::Result<()> {

        self.client.publish(&format!("{}.hello", &self.prefix), "Hello, world!").await?;
        Ok(())
    }
}