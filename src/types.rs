use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct User {
    pub id: Option<String>,
    pub username: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub avatar_url: Option<String>,
    pub signed_up: Option<String>,
    pub last_logged_in: Option<String>,
    pub status: Option<String>,
    pub developer_name: Option<String>,
    pub developer_website: Option<String>,
    pub developer_description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Score {
    pub score: Option<String>,
    pub sort: Option<String>,
    pub extra_data: Option<String>,
    pub user: Option<String>,
    pub user_id: Option<String>,
    pub guest: Option<String>,
    pub stored: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Trophy {
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub difficulty: Option<String>,
    pub image_url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_bool")]
    pub achieved: Option<bool>,
}

fn deserialize_optional_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::Bool(value)) => Ok(Some(value)),
        Some(serde_json::Value::String(value)) if value == "true" => Ok(Some(true)),
        Some(serde_json::Value::String(value)) if value == "false" => Ok(Some(false)),
        Some(value) => Err(serde::de::Error::custom(format!(
            "expected true/false string or boolean, got {value}"
        ))),
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ScoreTable {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub primary: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DataStoreItem {
    pub key: String,
    pub data: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Friend {
    pub id: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub status: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ServerTime {
    pub timestamp: Option<String>,
    pub timezone: Option<String>,
    pub year: Option<String>,
    pub month: Option<String>,
    pub day: Option<String>,
    pub hour: Option<String>,
    pub minute: Option<String>,
    pub second: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct UsersResponse {
    #[serde(default)]
    pub users: Vec<User>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ScoresResponse {
    #[serde(default)]
    pub scores: Vec<Score>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct TrophiesResponse {
    #[serde(default)]
    pub trophies: Vec<Trophy>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct TablesResponse {
    #[serde(default)]
    pub tables: Vec<ScoreTable>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct DataStoreResponse {
    pub data: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct DataStoreKeysResponse {
    #[serde(default)]
    pub keys: Vec<DataStoreItem>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct FriendsResponse {
    #[serde(default)]
    pub friends: Vec<Friend>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct TimeResponse {
    pub time: Option<ServerTime>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SessionResponse {
    pub session: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct MessageResponse {
    pub message: Option<String>,
}
