use gamejolt_rs::GameJoltClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = GameJoltClient::builder()
        .game_id(std::env::var("GAMEJOLT_GAME_ID")?)
        .private_key(std::env::var("GAMEJOLT_PRIVATE_KEY")?)
        .build()?;
    client
        .users()
        .authenticate(
            &std::env::var("GAMEJOLT_USERNAME")?,
            &std::env::var("GAMEJOLT_USER_TOKEN")?,
        )
        .await?;
    Ok(())
}
