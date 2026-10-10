# API reference

## Client construction

`GameJoltClient::builder()` returns a builder with SHA-1 selected by default.

| Builder method | Purpose |
| --- | --- |
| `game_id(value)` | Set the numeric game identifier as text. |
| `private_key(value)` | Set the signing secret. |
| `hash_algorithm(HashAlgo::Sha1)` | Select SHA-1 signing. |
| `hash_algorithm(HashAlgo::Md5)` | Select MD5 signing when enabled. |
| `base_url(value)` | Override the API root, primarily for tests. |
| `timeout(duration)` | Set a request timeout. |
| `build()` | Validate values and construct a reusable client. |

The client is cloneable. Clones share the underlying reqwest connection pool.
Its `Debug` implementation does not print the private key.

## Error results

Public operations return `gamejolt_rs::Result<T>`. The error enum is
`GameJoltError`; callers can pattern-match known cases and should include a
fallback arm because the enum is non-exhaustive.

| Variant | Meaning |
| --- | --- |
| `ApiError { code, message }` | Game Jolt rejected the API request. |
| `AuthError(message)` | Local or endpoint authentication failure. |
| `NetworkError(message)` | A transport failure occurred. |
| `SerializationError(message)` | A response or local value could not be decoded. |
| `RateLimited { retry_after }` | The caller has been rate-limited. |
| `OfflineQueueFull { capacity }` | An operation would exceed queue capacity. |
| `BatchError { count, errors }` | Batch-level aggregate failure. |
| `InvalidParameter { parameter, reason }` | Local input validation failed. |
| `Timeout { timeout }` | A request exceeded its configured deadline. |
| `SignatureError(message)` | A signature could not be generated. |
| `QueuePersistenceError(message)` | Queue bytes could not be read or written. |

Some variants reserve space for more advanced endpoints. Code should not
assume that every variant is currently produced by the implemented methods.

## Response envelope

Every endpoint response is expected to contain a `response` object and a
`success` field. The success field accepts either a JSON boolean or Game Jolt's
string representation `"true"` and `"false"`.

A missing envelope, missing success value, or unrecognized success value is a
serialization error. A failed API response becomes `ApiError`; the response
message is retained when present.

The decoder removes envelope metadata and deserializes the endpoint payload.
Unknown response fields are ignored by serde. Response structs are marked
non-exhaustive so new fields can be added without a breaking change.

## Users

Enable `users` and call `client.users()`.

### `authenticate(username, token)`

Checks that the player credentials are valid. Returns `Result<()>`.
Authentication makes a signed `users/auth` request. This method does not
return profile information; call `fetch` separately if the application needs
the user record.

### `fetch(username, user_id)`

Fetches users matching the supplied username or ID. Both values may be
omitted; prefer providing one filter to avoid requesting an unnecessarily
large result. Returns `Vec<User>`.

The response type includes optional profile data such as identifier,
username, account type, avatar, registration time, last-login time, status,
and developer profile details.

## Sessions

Enable `sessions` and call `client.sessions()`.

### `open(username, token)`

Opens a session for the authenticated player. The call returns after the API
has accepted the open operation.

### `ping(username, token, status)`

Keeps a session alive and sets its status. `SessionStatus::Active` means the
player is active. `SessionStatus::Idle` means the player is away or inactive.

### `close(username, token)`

Closes the player's session. Applications that do not use an automatic manager
should close sessions as part of their own orderly shutdown flow.

### `check(username, token)`

Checks for an active session and returns a boolean based on the API response.
An invalid response shape is reported as a serialization error.

### Automatic session manager

With `session-manager`, `auto_manage(username, token)` creates a builder.
`start().await` first opens the session and then starts a Tokio task.

`interval(duration)` controls ping cadence. Values below one second are clamped
to one second. The default interval is 30 seconds.

`retry_max(duration)` controls the upper bound on exponential retry delays.
The default maximum is 30 seconds.

The returned manager supports `set_status`, `subscribe`, and `stop().await`.
Dropping the manager signals the worker to close the session, but asynchronous
cleanup cannot be guaranteed if the runtime is already shutting down.

Events include open, ping, retry, close, and close-failure notifications.
Broadcast receivers can lag if they are not polled regularly.

## Scores

Enable `scores` and call `client.scores()`.

### Submit

`add(username, token)` starts a builder. Call `sort(i64)` to enter the ready
state. `score_str(text)`, `table(id)`, and `extra_data(text)` configure the
submission. `submit().await` performs the request.

The type-state transition prevents submission before a sort value is set.
When no display string is supplied, its decimal sort value is sent as the
display text.

### Fetch

`fetch(table_id, limit, guest)` returns `Vec<Score>`. Parameters are optional.
Use `table_id` to select a table and `limit` to bound the result size. A guest
name can be supplied for guest-oriented retrieval.

### Tables

`tables(trophy_id)` returns `Vec<ScoreTable>`. The trophy filter is optional.

### Rank

`rank(sort, table_id)` returns the optional rank for the given sort value.
The table identifier is optional.

## Trophies

Enable `trophies` and call `client.trophies()`.

### `fetch(username, token)`

Returns trophies and their achieved status for the account.

### `achieve(username, token, trophy_id)`

Marks a trophy as achieved for that user.

### `remove_achieved(username, token, trophy_id)`

Removes the achieved status for a trophy.

The `Trophy` response contains optional title, description, difficulty, image
URL, and an optional achieved boolean. Both JSON booleans and the string
representations `"true"` and `"false"` are accepted for the achieved field.

## Data store

Enable `datastore` and call `client.datastore()`.

### String values

`set(key, data)` writes a global key.

`update(key, data)` submits the API's data-store update request. This wrapper
currently sends the data string as provided; it does not implement a typed
server-side arithmetic or append operation.

`fetch(key)` returns a string value.

`remove(key)` deletes a global key.

`get_keys(pattern)` returns key/data records and optionally accepts a pattern.

### Serde values

`set_typed(key, &value)` serializes a value with `serde_json`.

`get_typed::<T>(key)` reads and deserializes the JSON value.

`update_typed(key, closure)` fetches, transforms, and writes a typed value. It
is not atomic. Concurrent game clients can overwrite each other's updates.

`set_typed_for_user(username, token, key, &value)` writes a user-scoped typed
entry. `get_typed_for_user` fetches and deserializes one.

Typed updates for user-scoped keys are not currently provided.

## Friends

Enable `friends` and call `client.friends().fetch(username, token)`. The method
returns `Vec<Friend>`. Friend fields are optional; treat missing names and
avatars as normal response cases.

## Server time

Enable `time` and call `client.time().fetch()`. The method returns
`Option<ServerTime>`. A time payload may contain timestamp, timezone, year,
month, day, hour, minute, and second fields.

## Batch operations

Enable `batch` and call `client.batch()`. Add `BatchOperation` values and call
`execute().await`. The current implementation runs each operation as an
individual signed request in ordered groups of at most 50 operations.

This is an ordered client-side batch interface, not a single network request.
It currently stops on the first failure and returns that error. It does not
yet produce independently typed result values for each heterogeneous
operation.

## Type inventory

| Type | Endpoint payload |
| --- | --- |
| `User` | User details |
| `Score` | Score entry |
| `Trophy` | Trophy details |
| `ScoreTable` | Score table details |
| `DataStoreItem` | Key and stored text |
| `Friend` | Friend profile details |
| `ServerTime` | Server time fields |
| `MessageResponse` | Optional endpoint message |
| `SessionStatus` | Active or idle session state |

All public response records are non-exhaustive. Prefer access through the
fields or methods available in the version you compile against, and do not
construct them outside the crate.
