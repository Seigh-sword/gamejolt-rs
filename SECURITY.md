# Security policy

## Supported versions

Security fixes are made against the current development version. Users should
upgrade to the latest release that is compatible with their application.

## Reporting a vulnerability

Please report security issues privately using GitHub's
[private vulnerability reporting](https://github.com/Seigh-sword/gamejolt-rs/security/advisories/new)
when it is enabled. If the feature is unavailable, contact the repository
maintainers privately through the contact information on the GitHub profile.

Do not open a public issue for an unpatched vulnerability.

Include the affected version, impact, reproduction steps, and a suggested
mitigation if known. Do not include real player data or credentials.

Maintainers will acknowledge reports as soon as practical, investigate them,
and coordinate a fix and disclosure timeline with the reporter. No response
time is guaranteed.

## Secrets and reports

Never send a Game Jolt private key, user token, signed request URL, or live
player data in an issue, pull request, or report. Redact logs before sharing.

The crate's client-side redaction does not protect secrets printed by
application code or stored in local offline queue files.
