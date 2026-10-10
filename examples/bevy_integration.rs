use bevy::prelude::App;
use gamejolt_rs::bevy_plugin::GameJoltPlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();
    app.add_plugins(GameJoltPlugin::new(
        std::env::var("GAMEJOLT_GAME_ID")?,
        std::env::var("GAMEJOLT_PRIVATE_KEY")?,
    )?);
    Ok(())
}
