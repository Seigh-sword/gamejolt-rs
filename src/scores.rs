use crate::client::{GameJoltClient, params};
use crate::error::Result;
use crate::types::{MessageResponse, Score, ScoresResponse, TablesResponse};
use std::marker::PhantomData;

pub struct ScoresEndpoint<'a> {
    client: &'a GameJoltClient,
}

impl<'a> ScoresEndpoint<'a> {
    pub(crate) fn new(client: &'a GameJoltClient) -> Self {
        Self { client }
    }

    pub fn add(&self, username: &str, token: &str) -> ScoreBuilder<'a, Missing> {
        ScoreBuilder {
            client: self.client,
            username: username.to_owned(),
            token: token.to_owned(),
            sort: 0,
            score: None,
            table_id: None,
            extra_data: None,
            state: PhantomData,
        }
    }

    pub async fn fetch(
        &self,
        table_id: Option<u32>,
        limit: Option<u32>,
        guest: Option<&str>,
    ) -> Result<Vec<Score>> {
        let mut values = Vec::new();
        if let Some(table_id) = table_id {
            values.extend(params(&[("table_id", &table_id.to_string())]));
        }
        if let Some(limit) = limit {
            values.extend(params(&[("limit", &limit.to_string())]));
        }
        if let Some(guest) = guest {
            values.extend(params(&[("guest", guest)]));
        }
        let response: ScoresResponse = self.client.request("scores", &values).await?;
        Ok(response.scores)
    }

    pub async fn tables(&self, trophy_id: Option<u32>) -> Result<Vec<crate::types::ScoreTable>> {
        let values = trophy_id
            .map(|id| params(&[("trophy_id", &id.to_string())]))
            .unwrap_or_default();
        let response: TablesResponse = self.client.request("scores/tables", &values).await?;
        Ok(response.tables)
    }

    pub async fn rank(&self, sort: i64, table_id: Option<u32>) -> Result<Option<u32>> {
        let mut values = params(&[("sort", &sort.to_string())]);
        if let Some(table_id) = table_id {
            values.extend(params(&[("table_id", &table_id.to_string())]));
        }
        let response: RankResponse = self.client.request("scores/get-rank", &values).await?;
        Ok(response.rank)
    }
}

pub struct Missing;
pub struct Ready;

pub struct ScoreBuilder<'a, State> {
    client: &'a GameJoltClient,
    username: String,
    token: String,
    sort: i64,
    score: Option<String>,
    table_id: Option<u32>,
    extra_data: Option<String>,
    state: PhantomData<State>,
}

impl<'a, State> ScoreBuilder<'a, State> {
    pub fn extra_data(mut self, value: impl Into<String>) -> Self {
        self.extra_data = Some(value.into());
        self
    }

    pub fn table(mut self, table_id: u32) -> Self {
        self.table_id = Some(table_id);
        self
    }
}

impl<'a, State> ScoreBuilder<'a, State> {
    pub fn sort(self, sort: i64) -> ScoreBuilder<'a, Ready> {
        ScoreBuilder {
            client: self.client,
            username: self.username,
            token: self.token,
            sort,
            score: self.score,
            table_id: self.table_id,
            extra_data: self.extra_data,
            state: PhantomData,
        }
    }
}

impl<'a> ScoreBuilder<'a, Ready> {
    pub fn score_str(mut self, score: impl Into<String>) -> Self {
        self.score = Some(score.into());
        self
    }

    pub async fn submit(self) -> Result<()> {
        let sort = self.sort;
        let score = self.score.unwrap_or_else(|| sort.to_string());
        let mut values = params(&[
            ("username", &self.username),
            ("user_token", &self.token),
            ("sort", &sort.to_string()),
            ("score", &score),
        ]);
        if let Some(table_id) = self.table_id {
            values.extend(params(&[("table_id", &table_id.to_string())]));
        }
        if let Some(extra_data) = self.extra_data {
            values.extend(params(&[("extra_data", &extra_data)]));
        }
        let _: MessageResponse = self.client.request("scores/add", &values).await?;
        Ok(())
    }
}

pub trait GameJoltScore {
    fn score_type_name() -> &'static str;
}

#[derive(serde::Deserialize)]
struct RankResponse {
    rank: Option<u32>,
}
