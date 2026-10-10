use gamejolt_rs::GameJoltClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = GameJoltClient::builder()
        .game_id(std::env::var("GAMEJOLT_GAME_ID")?)
        .private_key(std::env::var("GAMEJOLT_PRIVATE_KEY")?)
        .build()?;
    let session = client
        .sessions()
        .auto_manage(
            &std::env::var("GAMEJOLT_USERNAME")?,
            &std::env::var("GAMEJOLT_USER_TOKEN")?,
        )
        .start()
        .await?;
    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    session.stop().await?;
    Ok(())
}
