use crate::client::{GameJoltClient, params};
use crate::error::Result;
use crate::types::{Friend, FriendsResponse};

pub struct FriendsEndpoint<'a> {
    client: &'a GameJoltClient,
}

impl<'a> FriendsEndpoint<'a> {
    pub(crate) fn new(client: &'a GameJoltClient) -> Self {
        Self { client }
    }

    pub async fn fetch(&self, username: &str, token: &str) -> Result<Vec<Friend>> {
        let response: FriendsResponse = self
            .client
            .request(
                "friends",
                &params(&[("username", username), ("user_token", token)]),
            )
            .await?;
        Ok(response.friends)
    }
}
