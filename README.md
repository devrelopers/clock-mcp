# clock-mcp

[![crates.io](https://img.shields.io/crates/v/clock-mcp.svg)](https://crates.io/crates/clock-mcp)
[![CI](https://github.com/devrelopers/clock-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/devrelopers/clock-mcp/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

A Model Context Protocol server that gives an AI assistant a wall clock. It exposes five tools for reading the current time, doing duration math, and converting between IANA timezones, over MCP's stdio transport. Single Rust binary.

## Install

```sh
cargo install clock-mcp
```

This installs the `clock-mcp` binary to `~/.cargo/bin/`, which is on your `PATH` if you have a normal Rust setup.

Or build from source:

```sh
git clone https://github.com/devrelopers/clock-mcp.git
cd clock-mcp
cargo install --path .
```

## Use it with Claude Code

```sh
claude mcp add clock clock-mcp
```

Verify with `claude mcp list`.

## Use it with Claude Desktop

Add an entry to your `claude_desktop_config.json` (macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "clock": {
      "command": "clock-mcp"
    }
  }
}
```

Restart Claude Desktop. The five tools below will appear in the tools picker.

## Tool reference

### `now`

Returns the current wall-clock time.

**Params**

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `timezone` | `string` (IANA) | no | e.g. `"America/Denver"`. Defaults to `"UTC"`. |

**Example response**

```json
{
  "iso8601": "2026-04-20T12:03:26.639756-06:00",
  "unix_seconds": 1776708206,
  "timezone": "America/Denver"
}
```

### `time_until`

Returns the duration from now until a target datetime. Negative if the target is already in the past.

**Params**

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `target` | `string` (ISO 8601 / RFC 3339) | yes | e.g. `"2026-12-31T23:59:59Z"`. |

**Example response**

```json
{
  "from": "2026-04-20T18:03:26.649537+00:00",
  "to": "2026-12-31T23:59:59+00:00",
  "duration": {
    "total_seconds": 22053392,
    "days": 255,
    "hours": 5,
    "minutes": 56,
    "seconds": 32,
    "human": "255d 5h 56m 32s"
  }
}
```

### `time_since`

Returns the duration from a past datetime until now. Negative if the input is actually in the future.

**Params**

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `past` | `string` (ISO 8601 / RFC 3339) | yes | e.g. `"2026-01-01T00:00:00Z"`. |

**Example response**

```json
{
  "from": "2026-01-01T00:00:00+00:00",
  "to": "2026-04-20T18:03:26.659573+00:00",
  "duration": {
    "total_seconds": 9482606,
    "days": 109,
    "hours": 18,
    "minutes": 3,
    "seconds": 26,
    "human": "109d 18h 3m 26s"
  }
}
```

### `time_between`

Returns the duration between two datetimes. Negative if `end` precedes `start`.

**Params**

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `start` | `string` (ISO 8601 / RFC 3339) | yes | Start datetime. |
| `end` | `string` (ISO 8601 / RFC 3339) | yes | End datetime. |

**Example response**

```json
{
  "from": "2026-01-01T00:00:00+00:00",
  "to": "2026-01-02T01:30:00+00:00",
  "duration": {
    "total_seconds": 91800,
    "days": 1,
    "hours": 1,
    "minutes": 30,
    "seconds": 0,
    "human": "1d 1h 30m"
  }
}
```

### `convert_timezone`

Re-expresses an instant in another IANA timezone. The `unix_seconds` field is preserved across the conversion — only the wall-clock representation changes.

**Params**

| Name | Type | Required | Description |
| --- | --- | --- | --- |
| `datetime` | `string` (ISO 8601 / RFC 3339) | yes | Input datetime with a timezone offset. |
| `target_timezone` | `string` (IANA) | yes | e.g. `"Asia/Tokyo"`. |

**Example response**

```json
{
  "original": "2026-04-20T21:00:00+00:00",
  "converted": "2026-04-21T06:00:00+09:00",
  "target_timezone": "Asia/Tokyo",
  "unix_seconds": 1776718800
}
```

## Error shape

Every tool returns a structured JSON error (never a panic) on bad input. The payload has a required `error` string and an optional `hint`:

```json
{
  "error": "Unknown timezone: \"Narnia/Cair_Paravel\"",
  "hint": "Use an IANA timezone name like 'America/Denver', 'Europe/Berlin', 'Asia/Tokyo', or 'UTC'."
}
```

The MCP envelope around these errors sets `isError: true` on the `tools/call` result.

## Development

```sh
cargo build --release      # build the binary
cargo test                 # run unit tests
cargo clippy -- -D warnings
cargo fmt
```

### Poke it by hand

The server speaks MCP over stdio. You can talk to it with any MCP client, including the reference inspector:

```sh
npx @modelcontextprotocol/inspector clock-mcp
```

Or drive a raw JSON-RPC handshake yourself:

```sh
printf '%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"probe","version":"0"}}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
  | clock-mcp
```

## License

MIT. See [LICENSE](./LICENSE).
