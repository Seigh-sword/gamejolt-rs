use crate::GameJoltClient;
use crate::error::Result;
use futures::Stream;
use std::time::Duration;

pub fn watch_leaderboard(
    client: GameJoltClient,
    table_id: Option<u32>,
    interval: Duration,
) -> impl Stream<Item = Result<Vec<crate::types::Score>>> {
    async_stream::stream! {
        loop {
            yield client.scores().fetch(table_id, None, None).await;
            tokio::time::sleep(interval).await;
        }
    }
}
