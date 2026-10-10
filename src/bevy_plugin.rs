use crate::GameJoltClient;
use bevy::prelude::{App, Plugin, Resource};

#[derive(Resource, Clone)]
pub struct GameJoltResource(pub GameJoltClient);

pub struct GameJoltPlugin {
    client: GameJoltClient,
}

impl GameJoltPlugin {
    pub fn new(game_id: impl Into<String>, private_key: impl Into<String>) -> crate::Result<Self> {
        Ok(Self {
            client: GameJoltClient::builder()
                .game_id(game_id)
                .private_key(private_key)
                .build()?,
        })
    }
}

impl Plugin for GameJoltPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameJoltResource(self.client.clone()));
    }
}
