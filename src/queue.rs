use crate::db::EndpointKey;
use crate::error;

pub struct Queue {
    client: nats::asynk::Connection,
    prefix: String,
}

impl Queue {
    pub fn new(client: nats::asynk::Connection, prefix: String) -> Self {
        Queue { client, prefix }
    }

    fn subject(&self, topic: &str) -> String {
        format!("{}.{}", self.prefix, topic)
    }

    fn queue(&self, queue: &str) -> String {
        format!("{}.{}", self.prefix, queue)
    }

    pub async fn publish_hello_world(&self) -> std::io::Result<()> {
        self.client
            .publish(&self.subject("hello"), "Hello, world!")
            .await?;
        Ok(())
    }

    pub async fn publish_ping(&self, endpoint: EndpointKey) -> std::io::Result<()> {
        self.client
            .publish(&self.subject("ping"), endpoint.to_string())
            .await?;
        Ok(())
    }

    pub async fn subscribe_ping(
        &self,
    ) -> std::io::Result<impl futures_core::Stream<Item = error::Result<EndpointKey>>> {
        let sub = self
            .client
            .queue_subscribe(&self.subject("ping"), &self.queue("worker"))
            .await?;
        Ok(async_stream::try_stream! {
            while let Some(msg) = sub.next().await {
                let s = std::str::from_utf8(&msg.data).map_err(|_| error::Error::ParseEtcdKeyError)?;
                let key: EndpointKey = s.parse()?;
                yield key;
            }
        })
    }
}
