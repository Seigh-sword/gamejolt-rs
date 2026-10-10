use gamejolt_rs::GameJoltClient;

pub async fn connect(game_id: &str, private_key: &str) -> gamejolt_rs::Result<GameJoltClient> {
    GameJoltClient::builder()
        .game_id(game_id)
        .private_key(private_key)
        .build()
}

fn main() {}
