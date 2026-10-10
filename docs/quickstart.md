# Quickstart

## Requirements

- Rust 1.85 or a later stable compiler supporting edition 2024.
- A Game Jolt game identifier.
- A private key configured for that game.
- A user name and user token for authenticated player operations.

Do not use a user's password as a token. The game API uses its own user token
field and the account settings page exposes the appropriate credential.

## Add the dependency

The default features include users, sessions, scores, and SHA-1 signatures.
Select additional APIs in `Cargo.toml`:

```toml
[dependencies]
gamejolt-rs = { version = "0.1", features = ["trophies", "datastore"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Applications that already use Tokio should reuse their existing runtime
dependency instead of adding a second runtime.

## Configure credentials

Store credentials outside source control. For local development, environment
variables are convenient:

```text
GAMEJOLT_GAME_ID=your-game-id
GAMEJOLT_PRIVATE_KEY=your-private-key
GAMEJOLT_USERNAME=your-player-name
GAMEJOLT_USER_TOKEN=your-player-token
```

Use your deployment platform's secret store in production. The example files
read these variables at runtime and do not contain real keys.

## Build the client

```rust,no_run
use gamejolt_rs::{GameJoltClient, HashAlgo};

# fn main() -> gamejolt_rs::Result<()> {
let client = GameJoltClient::builder()
    .game_id(std::env::var("GAMEJOLT_GAME_ID").unwrap_or_default())
    .private_key(std::env::var("GAMEJOLT_PRIVATE_KEY").unwrap_or_default())
    .hash_algorithm(HashAlgo::Sha1)
    .build()?;
# let _ = client;
# Ok(())
# }
```

The builder validates that the game identifier and private key are present.
Configuration errors are returned from `build`; construction does not panic.
The builder defaults to SHA-1, which is the recommended choice for new
integrations.

Avoid defaulting failed environment lookups to empty strings in application
code. The compact snippet above keeps its focus on builder syntax; production
examples should propagate environment errors as shown in `examples/`.

## Authenticate a player

```rust,no_run
use gamejolt_rs::GameJoltClient;

# async fn run(client: &GameJoltClient) -> gamejolt_rs::Result<()> {
client.users().authenticate("player", "user-token").await?;
# Ok(())
# }
```

Successful authentication returns `Ok(())`. An invalid credential is surfaced
as an API error rather than being converted into a success-shaped value.

## Fetch player details

```rust,no_run
# async fn run(client: &gamejolt_rs::GameJoltClient) -> gamejolt_rs::Result<()> {
let users = client.users().fetch(Some("player"), None).await?;
for user in users {
    println!("{:?}", user.username);
}
# Ok(())
# }
```

User fields are optional because Game Jolt can omit details. Handle missing
values in the user interface instead of assuming every profile field exists.

## Submit a score

```rust,no_run
# async fn run(client: &gamejolt_rs::GameJoltClient) -> gamejolt_rs::Result<()> {
client
    .scores()
    .add("player", "user-token")
    .sort(1820)
    .score_str("1,820 stars")
    .table(1)
    .submit()
    .await?;
# Ok(())
# }
```

The sort value is the numeric value used for ordering. The score string is
human-readable text. A table identifier is optional when the game's primary
table should be used.

## Manage a session manually

```rust,no_run
# async fn run(client: &gamejolt_rs::GameJoltClient) -> gamejolt_rs::Result<()> {
let sessions = client.sessions();
sessions.open("player", "user-token").await?;
sessions.ping("player", "user-token", gamejolt_rs::sessions::SessionStatus::Active).await?;
sessions.close("player", "user-token").await?;
# Ok(())
# }
```

Use `Idle` while the player is away from the game. A manual session gives the
application complete control of the session lifecycle.

## Manage a session automatically

Enable `session-manager` along with `sessions`:

```rust,no_run
# async fn run(client: &gamejolt_rs::GameJoltClient) -> gamejolt_rs::Result<()> {
let manager = client
    .sessions()
    .auto_manage("player", "user-token")
    .interval(std::time::Duration::from_secs(30))
    .start()
    .await?;
manager.set_status(gamejolt_rs::sessions::SessionStatus::Idle);
manager.stop().await?;
# Ok(())
# }
```

The manager opens before it returns, pings on its configured interval, and
closes when `stop` is called or when the handle is dropped. Observe events with
`subscribe`. Automatic session management requires an active Tokio runtime.

## Configure signature algorithms

MD5 and SHA-1 code are separately feature-gated. The default includes `sha1`.
To use MD5:

```toml
gamejolt-rs = { version = "0.1", default-features = false, features = ["users", "md5"] }
```

Building with a hash algorithm whose implementation is disabled returns a
signature error. See [signing design](design.md#request-signatures).

## Local development

Run formatting and tests before publishing:

```text
cargo fmt --check
cargo test
cargo test --all-features
```

The all-feature test compiles optional integrations and is slower than the
default test. It does not contact the Game Jolt service.

## Next steps

- Read the [API reference](api-reference.md) for method details.
- Read [advanced usage](advanced.md) for queues and streams.
- Check [feature status](features.md) before depending on experimental modules.
- Review [security guidance](security.md) before deploying credentials.
