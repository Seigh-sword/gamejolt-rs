use crate::client::{GameJoltClient, params};
#[cfg(feature = "session-manager")]
use crate::error::GameJoltError;
use crate::error::Result;
use crate::types::MessageResponse;
#[cfg(feature = "session-manager")]
use std::time::Duration;
#[cfg(feature = "session-manager")]
use tokio::sync::{broadcast, oneshot, watch};
#[cfg(feature = "session-manager")]
use tokio::task::JoinHandle;

pub struct SessionsEndpoint<'a> {
    client: &'a GameJoltClient,
}

impl<'a> SessionsEndpoint<'a> {
    pub(crate) fn new(client: &'a GameJoltClient) -> Self {
        Self { client }
    }

    #[cfg(feature = "session-manager")]
    pub fn auto_manage(
        &self,
        username: impl Into<String>,
        token: impl Into<String>,
    ) -> AutoSessionBuilder {
        AutoSessionBuilder {
            client: self.client.clone(),
            username: username.into(),
            token: token.into(),
            interval: Duration::from_secs(30),
            retry_max: Duration::from_secs(30),
        }
    }

    pub async fn open(&self, username: &str, token: &str) -> Result<()> {
        let _: MessageResponse = self
            .client
            .request(
                "sessions/open",
                &params(&[("username", username), ("user_token", token)]),
            )
            .await?;
        Ok(())
    }

    pub async fn ping(&self, username: &str, token: &str, status: SessionStatus) -> Result<()> {
        let _: MessageResponse = self
            .client
            .request(
                "sessions/ping",
                &params(&[
                    ("username", username),
                    ("user_token", token),
                    ("status", status.as_str()),
                ]),
            )
            .await?;
        Ok(())
    }

    pub async fn close(&self, username: &str, token: &str) -> Result<()> {
        let _: MessageResponse = self
            .client
            .request(
                "sessions/close",
                &params(&[("username", username), ("user_token", token)]),
            )
            .await?;
        Ok(())
    }

    pub async fn check(&self, username: &str, token: &str) -> Result<bool> {
        let mut url = url::Url::parse(&format!("{}/sessions/check/", self.client.base_url))
            .map_err(|error| crate::error::GameJoltError::InternalError(error.to_string()))?;
        url.query_pairs_mut()
            .append_pair("username", username)
            .append_pair("user_token", token)
            .append_pair("game_id", &self.client.game_id)
            .append_pair("format", "json");
        let signature = self.client.signer.sign_url(url.as_str())?;
        url.query_pairs_mut().append_pair("signature", &signature);
        let value: serde_json::Value = self
            .client
            .http
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let response = value.get("response").ok_or_else(|| {
            crate::error::GameJoltError::SerializationError("missing response envelope".to_owned())
        })?;
        match response.get("success") {
            Some(serde_json::Value::String(value)) if value == "true" => Ok(true),
            Some(serde_json::Value::String(value)) if value == "false" => Ok(false),
            Some(serde_json::Value::Bool(value)) => Ok(*value),
            _ => Err(crate::error::GameJoltError::SerializationError(
                "response.success must be a boolean or the string true/false".to_owned(),
            )),
        }
    }
}

#[cfg(feature = "session-manager")]
pub struct AutoSessionBuilder {
    client: GameJoltClient,
    username: String,
    token: String,
    interval: Duration,
    retry_max: Duration,
}

#[cfg(feature = "session-manager")]
impl AutoSessionBuilder {
    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval.max(Duration::from_secs(1));
        self
    }

    pub fn retry_max(mut self, retry_max: Duration) -> Self {
        self.retry_max = retry_max.max(Duration::from_secs(1));
        self
    }

    pub async fn start(self) -> Result<SessionManager> {
        self.client
            .sessions()
            .open(&self.username, &self.token)
            .await?;
        let (cancel, mut cancel_rx) = oneshot::channel();
        let (status_tx, status_rx) = watch::channel(SessionStatus::Active);
        let (events_tx, _) = broadcast::channel(32);
        let events = events_tx.clone();
        let client = self.client;
        let username = self.username;
        let token = self.token;
        let interval = self.interval;
        let retry_max = self.retry_max;
        let task = tokio::spawn(async move {
            let _ = events.send(SessionEvent::Opened);
            let mut ticker = tokio::time::interval(interval);
            ticker.tick().await;
            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        let status = *status_rx.borrow();
                        let mut delay = Duration::from_secs(1);
                        loop {
                            match client.sessions().ping(&username, &token, status).await {
                                Ok(()) => {
                                    let _ = events.send(SessionEvent::Pinged(status));
                                    break;
                                }
                                Err(error) => {
                                    let _ = events.send(SessionEvent::Retrying(error.to_string()));
                                    tokio::select! {
                                        _ = tokio::time::sleep(delay) => {},
                                        _ = &mut cancel_rx => {
                                            match client.sessions().close(&username, &token).await {
                                                Ok(()) => {
                                                    let _ = events.send(SessionEvent::Closed);
                                                }
                                                Err(error) => {
                                                    let _ = events.send(SessionEvent::CloseFailed(error.to_string()));
                                                }
                                            }
                                            return;
                                        }
                                    }
                                    delay = delay.saturating_mul(2).min(retry_max);
                                }
                            }
                        }
                    }
                    _ = &mut cancel_rx => {
                        match client.sessions().close(&username, &token).await {
                            Ok(()) => {
                                let _ = events.send(SessionEvent::Closed);
                            }
                            Err(error) => {
                                let _ = events.send(SessionEvent::CloseFailed(error.to_string()));
                            }
                        }
                        return;
                    }
                }
            }
        });
        Ok(SessionManager {
            cancel: Some(cancel),
            task: Some(task),
            status: status_tx,
            events: events_tx,
        })
    }
}

#[cfg(feature = "session-manager")]
pub struct SessionManager {
    cancel: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
    status: watch::Sender<SessionStatus>,
    events: broadcast::Sender<SessionEvent>,
}

#[cfg(feature = "session-manager")]
impl SessionManager {
    pub fn set_status(&self, status: SessionStatus) {
        self.status.send_replace(status);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SessionEvent> {
        self.events.subscribe()
    }

    pub async fn stop(mut self) -> Result<()> {
        self.cancel.take();
        if let Some(task) = self.task.take() {
            task.await
                .map_err(|error| GameJoltError::InternalError(error.to_string()))?;
        }
        Ok(())
    }
}

#[cfg(feature = "session-manager")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionEvent {
    Opened,
    Pinged(SessionStatus),
    Retrying(String),
    Closed,
    CloseFailed(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionStatus {
    Active,
    Idle,
}

impl SessionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Idle => "idle",
        }
    }
}
