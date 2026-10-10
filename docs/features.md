# Feature status

Feature flags are opt-in except for the crate's default features. This page
separates implemented API surface from roadmap-level behavior. A feature name
does not imply every proposed integration detail is complete.

## Endpoint features

| Feature | Current status | Notes |
| --- | --- | --- |
| `users` | Implemented baseline | Authentication and user lookup. |
| `sessions` | Implemented baseline | Open, ping, close, and check. |
| `scores` | Implemented baseline | Add, fetch, tables, and rank. |
| `trophies` | Implemented baseline | Fetch, achieve, and remove achieved. |
| `datastore` | Implemented baseline | Global read/write/remove/key listing and typed values. |
| `friends` | Implemented baseline | Fetch friend list. |
| `time` | Implemented baseline | Fetch server time payload. |

Endpoint response coverage follows fields currently used by these methods.
It is not a promise that every field from the Game Jolt service is modeled.

## Core configuration

| Capability | Current status | Notes |
| --- | --- | --- |
| Reqwest connection pool | Implemented | Reused by cloned clients. |
| Builder pattern | Implemented | Requires a non-empty game ID and key. |
| SHA-1 signatures | Implemented | Enabled by default. |
| MD5 signatures | Implemented | Enable the `md5` feature. |
| Debug redaction | Implemented | Client, builder, and signer redact the private key. |
| Runtime abstraction | Not implemented | Client request and task APIs assume Tokio. |
| `Send + Sync` public API | Intended | Check specific target and optional feature combinations. |

## Advanced capabilities

| Feature | Current status | Important limitation |
| --- | --- | --- |
| `batch` | Ordered client-side interface | Does not send one HTTP batch request. |
| `offline-queue` | Persistent queue and explicit flush | Calls are not intercepted automatically. |
| `reactive` | Leaderboard snapshot polling | No event-delta stream or trophy watcher. |
| `rate-limiter` | Shared token-bucket request gate | Does not read service headers or return retry hints. |
| `session-manager` | Background Tokio open/ping/close task | Shutdown close depends on the Tokio runtime remaining alive. |
| `bevy` | Client resource plugin | No event types or asynchronous ECS systems yet. |
| `wasm` | Reserved feature flag | WASM support is not verified or guaranteed. |
| `macros` | Marker-trait derives | Does not generate datastore or score operations. |

These limitations are documented so applications can make explicit choices
instead of inferring behavior from feature names.

## Cargo feature relationships

- `default` enables `scores`, `users`, `sessions`, and `sha1`.
- `md5` enables the MD5 digest dependency.
- `sha1` enables the SHA-1 digest dependency.
- `offline-queue` enables bincode persistence.
- `reactive` enables async-stream and Tokio stream support plus scores.
- `session-manager` enables reactive support and sessions.
- `macros` enables the local derive crate.
- `bevy` enables Bevy plus session manager, queue, reactive, and limiter code.
- `full` enables the endpoint and advanced features declared in the manifest.

Always build with the exact feature set you intend to publish. A successful
default build does not validate optional dependencies or downstream macro
expansion.

## Compatibility expectations

The package follows semantic versioning beginning at `0.1`. Public records are
non-exhaustive. Additional optional fields may be added without requiring
external callers to construct response values.

Adding a feature should not silently change the default feature set. A
feature's dependencies should remain optional where possible. Tests should
include both minimal and all-feature builds.

## Roadmap quality gates

Before describing batch as server-side batching, verify the actual API wire
format and cover it with request-level tests.

Before describing the queue as automatic, define which operation classes are
safe to retry and how duplicate submissions are handled.

Before describing WASM support, compile for a supported `wasm32` target, run
browser tests, and ensure no server-only transport is selected.

Before describing the derive macros as code generators, test their generated
API in a downstream crate and document trait bounds and field behavior.

Before adding Bevy events, define how async task results are delivered into
the ECS world and how systems are cancelled during app shutdown.

## Reporting gaps

Feature status should be updated in the same change that implements a major
capability. Keep examples aligned with the API they demonstrate. A feature
that only compiles but has no behavioral tests should be described as
experimental rather than complete.

## Before release

Review every feature description against executable behavior.

Verify the API root and endpoint paths against the current Game Jolt protocol.

Run the mock tests using both default and all-feature configurations.

Confirm that examples compile with the feature flags listed in the manifest.

Check that the documentation does not imply hidden background retries.

Check that a queue flush reports failures and retains pending records.

Check that generated macro output is tested outside of the library crate.

Check WASM support on an installed target before advertising it as supported.
