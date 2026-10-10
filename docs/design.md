# Design notes

## Goals

The crate aims to provide typed asynchronous access to the Game Jolt Game API
v1_2 with explicit feature flags and errors.

Correct protocol behavior is more important than a large-looking method
surface. A convenience feature should not imply a guarantee the underlying
service does not provide.

The client should be reusable across tasks, endpoint methods should expose
domain-oriented parameters, and response types should tolerate forward
compatible fields.

## Client ownership

`GameJoltClient` owns a reqwest client, game ID, signer, and API root.
Reqwest's client is cheap to clone and shares its connection pool internally.

Endpoint values borrow the client for the duration of a call. They do not
retain credentials or spawn background work unless an explicit manager is
started.

The builder consumes configuration at `build`. Missing or blank required
values are reported as `InvalidParameter`.

## Request construction

Each request is built from an endpoint path and string key/value parameters.
The URL encoder produces the query representation. Game ID and JSON response
format parameters are then added.

The signer hashes the request URL with the private key appended. The
signature field is appended only after the digest is computed. The resulting
URL is issued through the reusable reqwest client.

Endpoint paths are internal constants in current modules. Custom base URLs
exist for tests and controlled deployments.

## Request signatures

The signer supports MD5 and SHA-1, selected by a non-exhaustive
`HashAlgorithm` enum. SHA-1 is the default algorithm.

The `sign` helper serializes parameter pairs using URL form encoding and
appends the secret before digesting. `sign_url` accepts an already constructed
URL and appends the secret before digesting.

The private key remains a private `String` field and is rendered as
`[REDACTED]` in debug output. The client builder also implements custom debug
formatting so a configured secret is never printed as ordinary text.

Cryptographic digest algorithms are provided by optional crates. Request
construction returns a typed signature error when the selected algorithm is
not compiled in.

## Response decoding

The HTTP body is first decoded as JSON. The common response decoder validates
the response envelope and interprets the success field.

Game Jolt's string booleans and JSON booleans are both recognized for the
success flag. Failed responses become API errors. Remaining payload fields
are deserialized into endpoint response structs.

All response structs are non-exhaustive and their individual fields are
optional unless a key itself is required to construct the type.

## Typestate score builder

The score builder has two marker states. `Missing` is returned from `add`.
Calling `sort` transitions to `Ready`. Only the ready state exposes `submit`.

This design catches a missing sort value at compile time. Optional display,
table, and extra data fields remain ordinary builder options.

The sort value is stored as an integer and converted to text at the request
boundary. The displayed score may be supplied independently.

## Error design

`GameJoltError` is non-exhaustive to allow additional domain variants.
Transport errors are converted to string-backed network or timeout errors so
the public error can be cloned for event delivery.

Serialization errors preserve a diagnostic message. API errors preserve the
response code and message. Applications should avoid showing raw diagnostic
strings directly to end users.

The error type implements `std::error::Error` through thiserror. Public
operations use the crate's `Result` alias.

## Async runtime

HTTP requests use reqwest's asynchronous API. Rate limiting and session
management use Tokio primitives. No async-std abstraction exists yet.

The runtime expectation should remain visible in feature documentation.
Adding a runtime-independent abstraction later requires isolating timers,
task spawning, synchronization, and stream integration behind a coherent
runtime layer.

## Endpoint feature flags

Each endpoint module is conditionally compiled. The default feature set stays
small and enables commonly used users, sessions, scores, and SHA-1 support.

Advanced feature dependencies should remain optional. `full` is a convenience
feature, not a recommendation for every binary or WASM build.

## Current boundaries

The batch interface does not encode multiple calls into one HTTP request.
Its name describes a grouped application interface only.

The queue is explicit; endpoint failures are not intercepted. Automatic
replay requires an operation policy to avoid duplicating writes.

Reactive support polls leaderboard snapshots. It does not establish a
websocket, subscribe to server events, or produce exact score deltas.

The Bevy plugin inserts a client resource. It does not hide async execution or
introduce event readers.

The derive macros implement marker traits only. They do not introspect
arbitrary field types or generate datastore accessors.

These boundaries should be revisited only with protocol research, focused
tests, and compatibility review.

## Extension guidelines

New endpoint methods should use shared request and parameter helpers.

New response records should tolerate omitted service fields and be marked
non-exhaustive.

New configuration values should validate at build time or return a typed
error at the operation boundary.

New background workers should expose explicit shutdown and observable failure
events. Destructors cannot await network operations.

New persistence should document consistency, encryption, corruption handling,
and concurrent-writer behavior.

New examples should be runnable, use environment variables for credentials,
and avoid comments or placeholder secrets that could be mistaken for real
credentials.
