# Contributing

Thanks for helping improve `gamejolt-rs`. This guide explains the development
setup, checks, and contribution workflow.

## Development setup

Install a stable Rust toolchain with Cargo. The crate uses Rust edition 2024.

Clone the repository, then run these commands from its root:

```text
cargo check
cargo test
cargo test --all-features
cargo fmt --check
```

`cargo test --all-features` checks optional integrations and takes longer.
Ordinary tests use local mocks and do not need Game Jolt credentials.

The project ignores `Cargo.lock` because it is a library crate. Cargo will
generate a local lockfile as needed.

`.gitattributes` normalizes text files to LF, `.editorconfig` describes common
editor formatting, and `.gitignore` excludes local build output, lockfiles,
credentials, and game save data. Issue and PR templates live under `.github/`;
Dependabot checks Cargo and Actions dependencies weekly.

## Making a change

1. Check existing issues and the [feature status](docs/features.md).
2. Keep the change focused and follow established module and feature patterns.
3. Verify endpoint details against the Game Jolt API protocol; do not infer
   request behavior from another SDK.
4. Add or update tests for the behavior.
5. Update the relevant API, feature, security, or usage documentation.
6. Run formatting, tests, and any feature-specific checks.
7. Open a pull request using the repository template.

For endpoint work, use the shared request and parameter helpers, expose a
feature-gated client accessor, test signed requests with a mock server, and
cover both successful and failed response envelopes.

Response structs should tolerate omitted service fields, use explicit serde
renames where needed, and remain non-exhaustive. Do not turn malformed data or
API failures into empty values or apparent success.

For background tasks, document runtime requirements and provide explicit
shutdown and observable error behavior. Destructors cannot await network
cleanup. Retry policy must account for whether an operation can safely be
repeated.

Keep feature dependencies optional where possible. Do not claim server-side
batching, automatic queue interception, push events, full WASM support, or
generated derive functionality unless the implementation and tests provide
those guarantees.

## Credentials and security

Never commit game private keys, user tokens, `.env` files, signed URLs, queue
data, or real credentials in examples and tests.

Network examples must read credentials from environment variables. Integration
tests against a real game must be explicitly ignored by default and require
dedicated test credentials.

Treat queued parameters and API error messages as sensitive or untrusted data.
Do not log authentication parameters. See [SECURITY.md](SECURITY.md) for
reporting vulnerabilities.

## Tests and quality checks

Run the smallest relevant test while iterating, followed by the full checks
before requesting review:

```text
cargo fmt --check
cargo test
cargo test --all-features
```

Endpoint changes should include mocked HTTP behavior, including URL parameters,
the signature, a successful response, and an API failure. Signing changes
should use known digest test vectors. Changes to macros should be tested from
an integration test that compiles generated code as a downstream consumer.

WASM checks may require installing `wasm32-unknown-unknown`; WASM support is
currently experimental. Run and report the exact target check when changing
WASM-specific code.

Do not add routine tests that call the production service. Avoid adding
dependencies unless existing crates cannot reasonably provide the required
behavior.

## Pull requests

Use a short, descriptive title. Explain the user-visible behavior, notable
implementation choices, feature flags affected, and tests run. Link related
issues and call out known limitations.

Keep pull requests focused. Include documentation when public behavior changes.
Do not include generated build output, local credentials, or unrelated
formatting changes.

The CI workflow runs formatting checks and default/all-feature tests. A green
CI result does not replace protocol review for API changes.

## Optional Git commit template

The repository includes `.gitmessage` as an optional shared commit-message
template. Enable it locally with:

```text
git config commit.template .gitmessage
```

This is intentionally a local setting. Do not commit a `.gitconfig` that
overrides contributors' Git identity or personal preferences.

## Optional AI review

Add the literal `!prompt` marker to the pull request description to request an
AI review. The workflow runs the small `qwen2.5-coder:1.5b` model locally in an
Ollama container on a GitHub-hosted CPU runner. It reads the pull request diff
using read-only access and writes its advisory review to the workflow job
summary; it does not post a comment or request write permissions.

The model runs in the GitHub Actions job rather than a hosted inference API.
The workflow downloads the model and Ollama container image at run time.
Reviews may be incomplete or incorrect; contributors and maintainers must
verify every suggestion. Never put secrets or private player data in a pull
request.

## Code of conduct and help

All participants are expected to follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
For usage questions, see [SUPPORT.md](SUPPORT.md). For vulnerabilities, use
the private reporting route in [SECURITY.md](SECURITY.md).
