use gamejolt_rs::offline_queue::{OfflineQueue, QueuedOperation};

#[tokio::main]
async fn main() -> gamejolt_rs::Result<()> {
    let queue = OfflineQueue::persistent(100, "save/queue.bin")?;
    queue
        .enqueue(QueuedOperation {
            endpoint: "scores/add".to_owned(),
            parameters: vec![("score".to_owned(), "5000".to_owned())],
        })
        .await?;
    Ok(())
}
