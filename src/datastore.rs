use crate::client::{GameJoltClient, params};
use crate::error::{GameJoltError, Result};
use crate::types::{DataStoreItem, DataStoreKeysResponse, DataStoreResponse, MessageResponse};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub struct DatastoreEndpoint<'a> {
    client: &'a GameJoltClient,
}

impl<'a> DatastoreEndpoint<'a> {
    pub(crate) fn new(client: &'a GameJoltClient) -> Self {
        Self { client }
    }

    pub async fn set(&self, key: &str, data: &str) -> Result<()> {
        self.write("set", key, data).await
    }

    pub async fn update(&self, key: &str, data: &str) -> Result<()> {
        self.write("update", key, data).await
    }

    pub async fn set_typed<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        self.set(key, &serde_json::to_string(value)?).await
    }

    pub async fn set_typed_for_user<T: Serialize>(
        &self,
        username: &str,
        token: &str,
        key: &str,
        value: &T,
    ) -> Result<()> {
        self.write_user("set", username, token, key, &serde_json::to_string(value)?)
            .await
    }

    pub async fn update_typed_value<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        self.update(key, &serde_json::to_string(value)?).await
    }

    pub async fn get_typed<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        let value = self.fetch(key).await?;
        serde_json::from_str(&value).map_err(Into::into)
    }

    pub async fn get_typed_for_user<T: DeserializeOwned>(
        &self,
        username: &str,
        token: &str,
        key: &str,
    ) -> Result<T> {
        let response: DataStoreResponse = self
            .client
            .request(
                "data-store",
                &params(&[("username", username), ("user_token", token), ("key", key)]),
            )
            .await?;
        let data = response.data.ok_or_else(|| GameJoltError::NotFound {
            resource: format!("datastore key {key}"),
        })?;
        serde_json::from_str(&data).map_err(Into::into)
    }

    pub async fn update_typed<T, F>(&self, key: &str, update: F) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce(T) -> T,
    {
        let current = self.get_typed(key).await?;
        let updated = update(current);
        self.set_typed(key, &updated).await?;
        Ok(updated)
    }

    pub async fn fetch(&self, key: &str) -> Result<String> {
        let response: DataStoreResponse = self
            .client
            .request("data-store", &params(&[("key", key)]))
            .await?;
        response.data.ok_or_else(|| GameJoltError::NotFound {
            resource: format!("datastore key {key}"),
        })
    }

    pub async fn remove(&self, key: &str) -> Result<()> {
        self.write("remove", key, "").await
    }

    pub async fn get_keys(&self, pattern: Option<&str>) -> Result<Vec<DataStoreItem>> {
        let values = pattern.map(|p| params(&[("pattern", p)]));
        let response: DataStoreKeysResponse = self
            .client
            .request("data-store/get-keys", &values.unwrap_or_default())
            .await?;
        Ok(response.keys)
    }

    async fn write(&self, operation: &str, key: &str, data: &str) -> Result<()> {
        let endpoint = format!("data-store/{operation}");
        let mut values = params(&[("key", key)]);
        if operation != "remove" {
            values.extend(params(&[("data", data)]));
        }
        let _: MessageResponse = self.client.request(&endpoint, &values).await?;
        Ok(())
    }

    async fn write_user(
        &self,
        operation: &str,
        username: &str,
        token: &str,
        key: &str,
        data: &str,
    ) -> Result<()> {
        let endpoint = format!("data-store/{operation}");
        let values = params(&[
            ("username", username),
            ("user_token", token),
            ("key", key),
            ("data", data),
        ]);
        let _: MessageResponse = self.client.request(&endpoint, &values).await?;
        Ok(())
    }
}

pub trait GameJoltDatastore: Serialize + DeserializeOwned {
    fn datastore_type_name() -> &'static str;
}
