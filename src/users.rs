use crate::client::{GameJoltClient, params};
use crate::error::Result;
use crate::types::{MessageResponse, User, UsersResponse};

pub struct UsersEndpoint<'a> {
    client: &'a GameJoltClient,
}

impl<'a> UsersEndpoint<'a> {
    pub(crate) fn new(client: &'a GameJoltClient) -> Self {
        Self { client }
    }

    pub async fn authenticate(&self, username: &str, token: &str) -> Result<()> {
        let _: MessageResponse = self
            .client
            .request(
                "users/auth",
                &params(&[("username", username), ("user_token", token)]),
            )
            .await?;
        Ok(())
    }

    pub async fn fetch_authenticated(&self, username: &str, token: &str) -> Result<User> {
        self.authenticate(username, token).await?;
        let response: UsersResponse = self
            .client
            .request("users", &params(&[("username", username)]))
            .await?;
        response
            .users
            .into_iter()
            .next()
            .ok_or_else(|| crate::error::GameJoltError::NotFound {
                resource: "authenticated user".to_owned(),
            })
    }

    pub async fn fetch(&self, username: Option<&str>, user_id: Option<&str>) -> Result<Vec<User>> {
        let mut values = Vec::new();
        if let Some(username) = username {
            values.extend(params(&[("username", username)]));
        }
        if let Some(user_id) = user_id {
            values.extend(params(&[("user_id", user_id)]));
        }
        let response: UsersResponse = self.client.request("users", &values).await?;
        Ok(response.users)
    }
}
