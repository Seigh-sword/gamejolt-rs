# Testing and troubleshooting

## Test layers

The crate uses focused unit tests for deterministic logic and local HTTP mocks
for request behavior. Tests must not require a real Game Jolt game or expose
real credentials.

Run the default test suite:

```text
cargo test
```

Compile all optional features and examples:

```text
cargo test --all-features
```

Check formatting:

```text
cargo fmt --check
```

Run these from the workspace root. When a change affects one endpoint, add a
focused integration test for the request and its response before relying on a
full build as evidence of correctness.

## Mock server tests

Use a local mock server and configure the client with its URL:

```rust,ignore
let server = wiremock::MockServer::start().await;
let client = gamejolt_rs::GameJoltClient::builder()
    .game_id("123")
    .private_key("local-test-key")
    .base_url(server.uri())
    .build()?;
```

Match the HTTP method and endpoint path. Match important query parameters
individually rather than comparing the entire URL string, because query
encoding and parameter ordering may change as the signer is improved.

Mock successful and failed envelopes. A success response should contain a
response object with the `success` field. Failure tests should verify that the
API error message and code are preserved.

Malformed envelopes should be tested for all of the following:

- No response object.
- A response object with no success value.
- An unsupported success value.
- A success value encoded as a JSON boolean.
- A success value encoded as a lowercase string.
- An invalid endpoint payload.

Never make mock tests accidentally contact the production host. Use an
assertion on the configured mock server's request count where possible.

## Signature tests

Digest tests should use published standard hash vectors rather than only
checking output length. Verify that a known URL plus a known private key
produces the expected digest.

Include characters that require URL encoding:

- Spaces.
- Ampersands.
- Equals signs.
- Unicode.
- Empty values.

The signer should sign exactly the URL representation sent over HTTP, except
for the signature query parameter itself. Tests must ensure that the signature
is not recursively included in its own input.

Check debug output for the client, builder, and signer. The private key must
not occur in any of those strings.

## Feature-matrix checks

The default build validates the primary user experience. The no-default
feature build checks that optional code does not leak into core modules.
All-features builds compile optional dependencies and examples.

Useful local commands include:

```text
cargo check --no-default-features
cargo check
cargo test
cargo test --all-features
cargo check --target wasm32-unknown-unknown --features wasm
```

The WASM check requires the target to be installed and may fail until browser
support is completed. A failed check must be reported as an unsupported target
or dependency issue, not hidden.

## API errors

An API error is distinct from a network error. It indicates the remote server
returned a valid response envelope with a failed status.

A network error indicates that the request could not complete or the response
could not be obtained. A timeout is surfaced as a timeout variant.

A serialization error indicates that a response body was received but could
not be parsed into the expected payload.

When reporting an issue, include:

- The endpoint family and feature set.
- The Rust toolchain version.
- The target platform.
- The error variant and message.
- A redacted example response shape.

Do not include private keys, user tokens, signatures, or raw authenticated
request URLs in issue reports.

## Common troubleshooting

### Builder reports an invalid parameter

Check that the game ID and private key are present and not whitespace-only.
Do not attempt to compensate for missing values by passing empty strings.

### The selected algorithm is unavailable

Enable `sha1` or `md5` in Cargo features. The crate intentionally reports a
signature error instead of substituting an algorithm.

### The server rejects a signature

Verify the game ID, private key, API root, and URL parameters. Ensure a proxy
has not changed query encoding or reordered parameters after signing.

Confirm that the private key is the game API key, not a user token. Avoid
manually appending a signature to the URL.

### A request returns a serialization error

Inspect a redacted mock response. Confirm the envelope has `response.success`
and the endpoint payload uses the expected field names. Unknown fields are
accepted; missing required fields may not be.

### A user profile contains no field

Response fields are optional. Treat absence as normal. A `None` field is
different from an invalid user credential.

### A queue flush keeps failing

The failing operation remains queued, as do operations after it. Inspect the
returned error, repair the cause, and flush again. Do not clear the queue
without first deciding whether the pending operations can be discarded.

### A reactive stream stops producing useful updates

The leaderboard watcher emits snapshots at a fixed interval. It is polling,
not a server push stream. Handle each error item and check that the runtime
continues polling.

### An automatic session does not close on process exit

Drop-triggered cleanup requires the Tokio runtime to remain alive long enough
for the worker to send its close request. Call `stop().await` during orderly
shutdown when closure must be observed.

## Continuous integration

CI should pin a minimum supported stable toolchain and a current stable
toolchain. Run formatting, default tests, all-features checks, and relevant
target checks separately so failures are easy to diagnose.

Network integration tests should be ignored by default and should require an
explicit environment variable. They should use a dedicated test game and must
not run against production player accounts.

The project currently does not include an automated CI workflow. Add one
alongside reproducible test commands before making a release commitment.
