# Advanced usage

## Client reuse

Construct one client per game configuration and clone it for tasks that need
independent ownership. Reqwest's client is internally shared and maintains a
connection pool. Avoid constructing a client for every request.

```rust,no_run
use gamejolt_rs::GameJoltClient;

# fn make_client() -> gamejolt_rs::Result<GameJoltClient> {
GameJoltClient::builder()
    .game_id("123")
    .private_key("loaded-from-secret-store")
    .build()
# }
```

Do not put a private key in logs, panic messages, metrics labels, or serialized
configuration. The client and builder format the key as redacted, but
application code still owns any copies passed into the builder.

## Custom API roots

`base_url` exists for local mock servers and compatible proxies:

```rust,no_run
# fn make_client() -> gamejolt_rs::Result<gamejolt_rs::GameJoltClient> {
gamejolt_rs::GameJoltClient::builder()
    .game_id("123")
    .private_key("test-secret")
    .base_url("http://127.0.0.1:8080/api/game/v1_2")
    .build()
# }
```

Do not configure a remote untrusted host. Requests include user tokens and the
signature value. A proxy must preserve URL query semantics and protect its
transport.

## Signing details

The signer hashes the request URL followed by the private key. Query strings
are encoded by the URL library before the signature is computed.

SHA-1 and MD5 are selected with Cargo features. A build that lacks the chosen
algorithm returns `SignatureError` instead of panicking or silently choosing
another algorithm.

Changing query parameter order changes the signed URL and therefore the
signature. Let the client construct signed URLs; do not hand-assemble signed
request URLs unless reproducing its encoding and ordering behavior exactly.

See [design notes](design.md#request-signatures) for the signing sequence.

## Rate limiting

Enable `rate-limiter` to use the token-bucket limiter:

```rust,no_run
use gamejolt_rs::{GameJoltClient, rate_limiter::RateLimiter};

# fn make_client() -> gamejolt_rs::Result<GameJoltClient> {
let mut client = GameJoltClient::builder()
    .game_id("123")
    .private_key("secret")
    .build()?;
client.set_rate_limit(RateLimiter::per_minute(120));
Ok(client)
# }
```

Each request acquires one token before it is sent. The limiter waits when
capacity is depleted, then refills at the configured average rate. A zero
capacity or zero refill duration is normalized to a usable minimum.

Use one limiter per shared client to apply one aggregate budget across cloned
client handles. The limiter is local policy; it cannot know the service's
current account-specific quota or server-side `Retry-After` headers.

## Offline queue

Enable `offline-queue` to create a queue:

```rust,no_run
use gamejolt_rs::offline_queue::{OfflineQueue, QueuedOperation};

# async fn run(client: &gamejolt_rs::GameJoltClient) -> gamejolt_rs::Result<()> {
let queue = OfflineQueue::new(256);
queue.enqueue(QueuedOperation {
    endpoint: "scores/add".to_owned(),
    parameters: vec![("sort".to_owned(), "5000".to_owned())],
}).await?;
client.flush_offline_queue(&queue).await?;
# Ok(())
# }
```

`persistent(capacity, path)` loads or creates a bincode-serialized queue file.
The queue enforces its configured capacity. `pending()` returns the number of
waiting operations.

Flushing is explicit. The client does not automatically intercept failed
endpoint calls and enqueue them. Applications decide whether an operation is
safe to retry and construct queue records accordingly.

The flush processes operations in order. On the first failure, the failed
operation and all later operations are restored to the front of the queue.
The error is returned to the caller. A successful flush returns the count.

Do not enqueue credentials or secret material in plaintext. Queue persistence
is not encrypted, and queued parameters are stored verbatim. Avoid putting
user tokens in the queue or protect the storage using the platform's secure
storage facilities.

## Reactive polling

Enable `reactive` for a leaderboard stream:

```rust,no_run
use futures::StreamExt;
use std::time::Duration;

# async fn run(client: gamejolt_rs::GameJoltClient) {
let mut stream = gamejolt_rs::reactive::watch_leaderboard(
    client,
    Some(1),
    Duration::from_secs(60),
);
if let Some(snapshot) = stream.next().await {
    match snapshot {
        Ok(scores) => println!("received {} scores", scores.len()),
        Err(error) => eprintln!("leaderboard polling failed: {error}"),
    }
}
# }
```

The stream yields a complete score snapshot at each interval, not a delta.
Repeated identical values may be emitted. Polling is not push notification and
does not guarantee that every transient leaderboard change is observed.

Consumers should handle each `Err` item and continue polling if appropriate.
Dropping the stream stops future polling.

## Batching interface

`BatchBuilder` accepts `BatchOperation` values containing an endpoint and
owned query parameters. `len` and `is_empty` are useful before execution.

Execution currently makes a sequence of signed requests. Groups of fifty are
an iteration boundary, not a wire-level batch request. Do not use this API to
claim one network round trip or expect partial success results.

For parallel read-only fan-out, create independent futures and bound their
concurrency at the application level. For state-changing operations, retain
ordered execution unless operations are known to be independent.

## Bevy

Enable `bevy` to add `GameJoltPlugin`:

```rust,no_run
use bevy::prelude::App;
use gamejolt_rs::bevy_plugin::GameJoltPlugin;

# fn setup() -> gamejolt_rs::Result<()> {
let mut app = App::new();
app.add_plugins(GameJoltPlugin::new("123", "secret")?);
# Ok(())
# }
```

The plugin inserts a cloneable `GameJoltResource` that wraps the client. It
does not currently register score or trophy event types, run async systems, or
manage game sessions automatically.

Bevy's ECS scheduling is synchronous by default. Applications need an
explicit asynchronous integration strategy for network calls.

## Derive macros

Enable `macros` to use `GameJoltDatastore` and `GameJoltScore` derives:

```rust,ignore
#[derive(gamejolt_rs::GameJoltDatastore)]
struct PlayerSave {
    stage: u32,
}
```

The current derives implement marker traits exposing the type name. They do
not generate save/load methods, field mappings, or score serialization.
Implement serialization and endpoint calls explicitly until those derive
capabilities are expanded.

## WASM target

The `wasm` feature is reserved for browser-target configuration. The current
crate manifest does not promise that every endpoint or integration compiles
to `wasm32-unknown-unknown`. Verify the exact target and dependency graph for
your application before shipping.

Never embed a Game Jolt private key in a browser or WASM artifact. The key can
be recovered by users. Route privileged calls through a trusted backend or
only use operations designed to be safe without a private key.
