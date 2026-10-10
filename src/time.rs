use crate::client::GameJoltClient;
use crate::error::Result;
use crate::types::{ServerTime, TimeResponse};

pub struct TimeEndpoint<'a> {
    client: &'a GameJoltClient,
}

impl<'a> TimeEndpoint<'a> {
    pub(crate) fn new(client: &'a GameJoltClient) -> Self {
        Self { client }
    }

    pub async fn fetch(&self) -> Result<Option<ServerTime>> {
        let response: TimeResponse = self.client.request("time", &[]).await?;
        Ok(response.time)
    }
}
