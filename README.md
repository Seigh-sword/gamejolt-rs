# gamejolt-rs

[![CI](https://github.com/Seigh-sword/gamejolt-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Seigh-sword/gamejolt-rs/actions/workflows/ci.yml)
[![Opt-in AI review](https://github.com/Seigh-sword/gamejolt-rs/actions/workflows/ai-review.yml/badge.svg)](https://github.com/Seigh-sword/gamejolt-rs/actions/workflows/ai-review.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)

An async Rust client for the Game Jolt Game API v1_2.

This project is being built in public-sized pieces. The core client, signatures,
user/session/score/trophy/datastore/friend/time calls, and several opt-in
utilities are implemented. Some features described as future goals are not yet
production-complete; the feature matrix in [the feature guide](docs/features.md)
marks their current limits explicitly.

## Quick start

```rust,no_run
use gamejolt_rs::GameJoltClient;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let client = GameJoltClient::builder()
    .game_id(std::env::var("GAMEJOLT_GAME_ID")?)
    .private_key(std::env::var("GAMEJOLT_PRIVATE_KEY")?)
    .build()?;

client
    .users()
    .authenticate("player-name", "player-token")
    .await?;
# Ok(())
# }
```

The private key is only used locally to sign requests. It is redacted in the
debug representations of the client, builder, and signer. Keep the key on a
trusted server; shipping it inside a public client binary makes the game
credentials recoverable.

## Features

By default the crate enables `scores`, `users`, `sessions`, and `sha1`.
Enable other endpoint modules and optional integrations explicitly:

```toml
[dependencies]
gamejolt-rs = { version = "0.1", features = ["trophies", "datastore"] }
```

The `full` feature enables all currently declared modules, including both
hash implementations and integrations. See [features](docs/features.md) before
selecting experimental modules.

## API map

| API family | Client entry point | Feature |
| --- | --- | --- |
| Player authentication and lookup | `client.users()` | `users` |
| Sessions | `client.sessions()` | `sessions` |
| Scores and tables | `client.scores()` | `scores` |
| Trophies | `client.trophies()` | `trophies` |
| Data store | `client.datastore()` | `datastore` |
| Friends | `client.friends()` | `friends` |
| Server time | `client.time()` | `time` |

The [API reference](docs/api-reference.md) describes each implemented method
and parameter. The [advanced guide](docs/advanced.md) covers signing,
rate-limiting, queues, streaming, and integration boundaries.

## Score submission

The score builder requires a sort value before submission:

```rust,no_run
# async fn run(client: &gamejolt_rs::GameJoltClient) -> gamejolt_rs::Result<()> {
client
    .scores()
    .add("player-name", "player-token")
    .sort(5000)
    .score_str("5,000 points")
    .table(1)
    .submit()
    .await
# }
```

The builder uses a typestate transition so `.submit()` is available only after
`.sort(...)` has been called. The display string is optional and defaults to
the decimal representation of the sort value.

## Datastore values

Values can be serialized using serde:

```rust,no_run
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SaveData {
    level: u32,
    unlocked: Vec<String>,
}

# async fn run(client: &gamejolt_rs::GameJoltClient) -> gamejolt_rs::Result<()> {
let data = SaveData { level: 3, unlocked: vec![] };
client.datastore().set_typed("save", &data).await?;
let restored: SaveData = client.datastore().get_typed("save").await?;
# let _ = restored;
# Ok(())
# }
```

`update_typed` is a read-transform-write convenience, not a server-side atomic
transaction. Use it only when concurrent writers cannot overwrite one
another's changes.

## Documentation set

- [Quickstart](docs/quickstart.md)
- [API reference](docs/api-reference.md)
- [Advanced usage](docs/advanced.md)
- [Feature status](docs/features.md)
- [Testing and troubleshooting](docs/testing.md)
- [Security and operations](docs/security.md)
- [Design notes](docs/design.md)
- [Contributing](CONTRIBUTING.md)

All examples that make network calls read credentials from environment
variables. Network examples are not run by ordinary unit-test commands.

## License

Apache-2.0. See the repository license metadata for the full license text.
