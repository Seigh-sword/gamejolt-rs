# Security and operations

## Private key protection

The Game Jolt private key is a game-level secret. Anyone who extracts it from
a shipped desktop binary, mobile application, or browser bundle can create
requests that appear to come from the game.

Do not embed the private key in:

- Browser applications.
- WASM modules distributed to players.
- Public source repositories.
- Crash reports or telemetry.
- Debug output or structured log fields.
- Screenshots, issue reports, or support tickets.

For public clients, move privileged operations to a trusted backend. Use
short-lived player credentials and server-side checks where possible.

The library redacts the private key from its own `Debug` implementations.
Application-level wrappers, builders, or configuration structs can still
leak keys if their own debug formatting prints stored fields.

## User tokens

Treat player tokens as credentials. Send them only to the expected Game Jolt
API host or a trusted proxy. Avoid recording complete request URLs because
tokens and signatures are query parameters in this API.

Keep logs focused on endpoint names, request identifiers, elapsed time, and
error categories. Do not log parameter maps for authenticated calls.

The offline queue stores parameter strings without encryption. Do not include
user tokens in queued operations unless the storage is protected and the
threat model explicitly permits it.

## Transport and proxies

The default client uses HTTPS with rustls on native targets. A custom API root
is intended primarily for local tests or controlled infrastructure.

Use HTTPS for every remote endpoint. Do not disable certificate validation.
Do not direct authenticated requests to a URL supplied by untrusted input.

A proxy that terminates TLS must be trusted with all request data. Confirm its
logging, retention, access control, and URL rewriting policies.

## Signature integrity

The signature protects request integrity according to the Game Jolt API
protocol; it does not hide query parameters. It is not an encryption scheme.

Do not reuse a signature as an authentication token. Do not expose signed
URLs in analytics or browser referrers.

Signing an altered URL produces a different digest. Construct parameters
before issuing the request and let the client generate the signature for the
final URL.

## Retries and idempotency

Retrying a read request is usually safe. Retrying a write may duplicate a score
or repeat a state transition depending on the service's behavior.

The session manager retries failed pings with exponential delay. The offline
queue retries failed operations after a later flush. Applications should
choose queue contents conservatively and understand whether each operation
can be repeated.

Do not automatically queue every network failure. A malformed request,
invalid credential, or API-level rejection is not an outage and may never
succeed on retry.

## Queue file handling

The persistent queue serializes records to a local file. Protect the file with
operating-system permissions and consider encryption at rest when it can
contain sensitive data.

Concurrent writers to one queue path are not coordinated across processes.
Use one owner or an external synchronization strategy.

Back up or remove queue files only after confirming that pending operations
are no longer required. Deleting a queue can lose score submissions or game
state changes.

## Error handling

Errors can contain server-provided messages. Treat them as untrusted display
text. Escape messages before rendering them in HTML or a game UI.

Avoid success-shaped fallbacks. If a request fails, surface the failure to the
calling layer, then let the application choose whether to retry, report it, or
continue offline.

Do not expose raw network errors to players if they include URLs or other
implementation details. Keep a safe user message separate from diagnostic
logs.

## Threat-model checklist

- Is the private key ever distributed to an untrusted client?
- Are user tokens included in logs or telemetry?
- Can untrusted input influence the API base URL?
- Is persistent queue content sensitive?
- Are retries appropriate for the specific operation?
- Are server error messages safely escaped?
- Can an attacker trigger unbounded queued work?
- Is a per-client rate limit shared across clones?
- Can user-facing behavior distinguish offline state from rejected requests?
- Are secrets excluded from crash dumps where practical?

Review these questions when adding endpoint methods, persistence formats, or
new integrations.

## Incident response

If a game private key is exposed, rotate it using the Game Jolt dashboard and
update all trusted deployments. Review logs for unexpected use and do not
assume deleting the exposed copy is sufficient.

If a player token is exposed, follow the account and service procedures for
invalidating it. Remove exposed artifacts from logs and support channels where
possible, while preserving any data needed for an authorized security
investigation.

## Operational monitoring

Track API latency by endpoint family without labeling metrics with usernames.

Count network failures separately from API rejections.

Measure queue depth and age without exporting raw queued parameters.

Alert when pending operations remain unflushed beyond the game's expected
offline window.

Avoid logging complete query strings when instrumenting HTTP requests.

Use a correlation identifier that is generated locally and does not contain
player credentials.

Review dependency updates for transport and serialization security fixes.

Rotate game credentials after staff changes that affected secret access.

Document the owner and rotation procedure for every deployment environment.

Keep development credentials separate from production game credentials.
