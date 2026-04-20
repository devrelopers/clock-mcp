# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-04-20

### Added
- Initial release.
- MCP server over stdio transport, built on `rmcp` 1.5.
- `now` tool — returns current time in a given IANA timezone (defaults to UTC).
- `time_until` tool — duration from now to a target datetime, signed.
- `time_since` tool — duration from a past datetime to now, signed.
- `time_between` tool — duration between two datetimes, signed.
- `convert_timezone` tool — re-expresses an instant in another IANA timezone.
- Structured `{ error, hint }` JSON error shape on bad input.
- CI on GitHub Actions: fmt, clippy, test, release build, publish dry-run.

[Unreleased]: https://github.com/devrelopers/clock-mcp/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/devrelopers/clock-mcp/releases/tag/v0.1.0
