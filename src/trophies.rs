use crate::client::{GameJoltClient, params};
use crate::error::Result;
use crate::types::{MessageResponse, TrophiesResponse, Trophy};

pub struct TrophiesEndpoint<'a> {
    client: &'a GameJoltClient,
}

impl<'a> TrophiesEndpoint<'a> {
    pub(crate) fn new(client: &'a GameJoltClient) -> Self {
        Self { client }
    }

    pub async fn fetch(&self, username: &str, token: &str) -> Result<Vec<Trophy>> {
        let response: TrophiesResponse = self
            .client
            .request(
                "trophies",
                &params(&[("username", username), ("user_token", token)]),
            )
            .await?;
        Ok(response.trophies)
    }

    pub async fn achieve(&self, username: &str, token: &str, trophy_id: u32) -> Result<()> {
        self.change_achievement("trophies/add-achieved", username, token, trophy_id)
            .await
    }

    pub async fn remove_achieved(&self, username: &str, token: &str, trophy_id: u32) -> Result<()> {
        self.change_achievement("trophies/remove-achieved", username, token, trophy_id)
            .await
    }

    async fn change_achievement(
        &self,
        endpoint: &str,
        username: &str,
        token: &str,
        trophy_id: u32,
    ) -> Result<()> {
        let _: MessageResponse = self
            .client
            .request(
                endpoint,
                &params(&[
                    ("username", username),
                    ("user_token", token),
                    ("trophy_id", &trophy_id.to_string()),
                ]),
            )
            .await?;
        Ok(())
    }
}
