use crate::error::{GameJoltError, Result};
#[cfg(feature = "rate-limiter")]
use crate::rate_limiter::RateLimiter;
use crate::signing::{HashAlgorithm, Signer};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::fmt;

const API_ROOT: &str = "https://api.gamejolt.com/api/game/v1_2";

#[derive(Clone)]
pub struct GameJoltClient {
    pub(crate) http: reqwest::Client,
    pub(crate) game_id: String,
    pub(crate) signer: Signer,
    pub(crate) base_url: String,
    #[cfg(feature = "rate-limiter")]
    rate_limiter: Option<RateLimiter>,
}

impl fmt::Debug for GameJoltClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GameJoltClient")
            .field("game_id", &self.game_id)
            .field("signer", &self.signer)
            .field("base_url", &self.base_url)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashAlgo {
    Md5,
    Sha1,
}

impl From<HashAlgo> for HashAlgorithm {
    fn from(value: HashAlgo) -> Self {
        match value {
            HashAlgo::Md5 => Self::Md5,
            HashAlgo::Sha1 => Self::Sha1,
        }
    }
}

#[derive(Clone)]
pub struct GameJoltClientBuilder {
    game_id: Option<String>,
    private_key: Option<String>,
    hash_algorithm: HashAlgorithm,
    base_url: String,
    timeout: std::time::Duration,
}

impl fmt::Debug for GameJoltClientBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GameJoltClientBuilder")
            .field("game_id", &self.game_id)
            .field(
                "private_key",
                &self.private_key.as_ref().map(|_| "[REDACTED]"),
            )
            .field("hash_algorithm", &self.hash_algorithm)
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl Default for GameJoltClientBuilder {
    fn default() -> Self {
        Self {
            game_id: None,
            private_key: None,
            hash_algorithm: HashAlgorithm::Sha1,
            base_url: API_ROOT.to_owned(),
            timeout: std::time::Duration::from_secs(30),
        }
    }
}

impl GameJoltClient {
    pub fn builder() -> GameJoltClientBuilder {
        GameJoltClientBuilder::default()
    }

    pub fn game_id(&self) -> &str {
        &self.game_id
    }

    #[cfg(feature = "offline-queue")]
    pub async fn flush_offline_queue(
        &self,
        queue: &crate::offline_queue::OfflineQueue,
    ) -> Result<usize> {
        let _flush_guard = queue.begin_flush().await;
        let mut completed = 0;
        while let Some(operation) = queue.next().await {
            self.request::<Value>(&operation.endpoint, &operation.parameters)
                .await?;
            queue.acknowledge().await?;
            completed += 1;
        }
        Ok(completed)
    }

    #[cfg(feature = "rate-limiter")]
    pub fn set_rate_limit(&mut self, limiter: RateLimiter) {
        self.rate_limiter = Some(limiter);
    }

    #[cfg(feature = "batch")]
    pub fn batch(&self) -> crate::batch::BatchBuilder<'_> {
        crate::batch::BatchBuilder::new(self)
    }

    #[cfg(feature = "users")]
    pub fn users(&self) -> crate::users::UsersEndpoint<'_> {
        crate::users::UsersEndpoint::new(self)
    }

    #[cfg(feature = "sessions")]
    pub fn sessions(&self) -> crate::sessions::SessionsEndpoint<'_> {
        crate::sessions::SessionsEndpoint::new(self)
    }

    #[cfg(feature = "scores")]
    pub fn scores(&self) -> crate::scores::ScoresEndpoint<'_> {
        crate::scores::ScoresEndpoint::new(self)
    }

    #[cfg(feature = "trophies")]
    pub fn trophies(&self) -> crate::trophies::TrophiesEndpoint<'_> {
        crate::trophies::TrophiesEndpoint::new(self)
    }

    #[cfg(feature = "datastore")]
    pub fn datastore(&self) -> crate::datastore::DatastoreEndpoint<'_> {
        crate::datastore::DatastoreEndpoint::new(self)
    }

    #[cfg(feature = "friends")]
    pub fn friends(&self) -> crate::friends::FriendsEndpoint<'_> {
        crate::friends::FriendsEndpoint::new(self)
    }

    #[cfg(feature = "time")]
    pub fn time(&self) -> crate::time::TimeEndpoint<'_> {
        crate::time::TimeEndpoint::new(self)
    }

    pub(crate) async fn request<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(String, String)],
    ) -> Result<T> {
        #[cfg(feature = "rate-limiter")]
        if let Some(limiter) = &self.rate_limiter {
            limiter.acquire().await;
        }
        let path = format!("{endpoint}/");
        let mut url = url::Url::parse(&format!("{}/{path}", self.base_url))
            .map_err(|error| GameJoltError::InternalError(error.to_string()))?;
        {
            let mut query = url.query_pairs_mut();
            for (key, value) in params {
                query.append_pair(key, value);
            }
            query.append_pair("game_id", &self.game_id);
            query.append_pair("format", "json");
        }
        let signature = self.signer.sign_url(url.as_str())?;
        url.query_pairs_mut().append_pair("signature", &signature);
        let response: Value = self
            .http
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        decode_response(response)
    }
}

impl GameJoltClientBuilder {
    pub fn game_id(mut self, game_id: impl Into<String>) -> Self {
        self.game_id = Some(game_id.into());
        self
    }

    pub fn private_key(mut self, private_key: impl Into<String>) -> Self {
        self.private_key = Some(private_key.into());
        self
    }

    pub fn hash_algorithm(mut self, algorithm: HashAlgo) -> Self {
        self.hash_algorithm = algorithm.into();
        self
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_owned();
        self
    }

    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn build(self) -> Result<GameJoltClient> {
        let game_id = self
            .game_id
            .filter(|id| !id.trim().is_empty())
            .ok_or_else(|| GameJoltError::InvalidParameter {
                parameter: "game_id".to_owned(),
                reason: "must not be empty".to_owned(),
            })?;
        let private_key = self
            .private_key
            .filter(|key| !key.trim().is_empty())
            .ok_or_else(|| GameJoltError::InvalidParameter {
                parameter: "private_key".to_owned(),
                reason: "must not be empty".to_owned(),
            })?;
        let http = reqwest::Client::builder().timeout(self.timeout).build()?;
        Ok(GameJoltClient {
            http,
            game_id,
            signer: Signer::new(private_key, self.hash_algorithm),
            base_url: self.base_url,
            #[cfg(feature = "rate-limiter")]
            rate_limiter: None,
        })
    }
}

pub(crate) fn decode_response<T: DeserializeOwned>(value: Value) -> Result<T> {
    let response = value
        .get("response")
        .and_then(Value::as_object)
        .ok_or_else(|| GameJoltError::SerializationError("missing response envelope".to_owned()))?;
    let success = response
        .get("success")
        .ok_or_else(|| {
            GameJoltError::SerializationError("missing response.success field".to_owned())
        })
        .and_then(parse_success)?;
    if !success {
        return Err(GameJoltError::ApiError {
            code: response
                .get("code")
                .and_then(Value::as_i64)
                .unwrap_or_default() as i32,
            message: response
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("request failed")
                .to_owned(),
        });
    }
    let mut payload = Map::new();
    for (key, value) in response {
        if key != "success" && key != "message" && key != "code" {
            payload.insert(key.clone(), value.clone());
        }
    }
    serde_json::from_value(Value::Object(payload)).map_err(Into::into)
}

fn parse_success(value: &Value) -> Result<bool> {
    match value {
        Value::Bool(value) => Ok(*value),
        Value::String(value) if value == "true" => Ok(true),
        Value::String(value) if value == "false" => Ok(false),
        _ => Err(GameJoltError::SerializationError(
            "response.success must be a boolean or the string true/false".to_owned(),
        )),
    }
}

pub(crate) fn params(values: &[(&str, &str)]) -> Vec<(String, String)> {
    values
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}
