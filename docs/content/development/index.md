# Development

This page explains the repository layout and the commands that build, check, and test Cuhlumn.

## Repository layout

| Path | Contents |
| --- | --- |
| `crates/api` | The `cuhlumn-api` library with the wire types and the validated value types |
| `crates/server` | The `cuhlumn-server` binary: HTTP API, public blog pages, share images, and the embedded console |
| `crates/cli` | The `cuhlumn` command-line client |
| `console/` | The management console, a Preact single-page app |
| `docs/` | This documentation site, built with Zensical |
| `skills/cuhlumn/` | The agent skill for the `cuhlumn` CLI |
| `.agents/skills/` | Skills for agents that work on this repository |

`AGENTS.md` at the repository root contains the code rules for Rust, the console, and written output.

## Requirements

- Stable Rust with edition 2024 support
- Node.js 22.12 or later and pnpm 11
- uv, only for the documentation site

## Rust commands

Run these commands from the repository root:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p cuhlumn-server
```

The integration tests send requests through the router against a temporary data folder.
They do not need a network connection.

## Console commands

Run these commands from the repository root:

```sh
pnpm install
pnpm dev                 # console at http://localhost:5173
pnpm validate            # Oxlint and the TypeScript check
pnpm test                # Vitest
pnpm format              # Oxfmt
pnpm build               # production build in console/dist
```

The development server sends `/api` to `http://127.0.0.1:8080` and keeps the `Host` header.
Start the server with these settings during console development:

```sh
CUHLUMN_TOKEN=<at least 32 characters> \
CUHLUMN_CONSOLE_HOST=localhost \
CUHLUMN_SCHEME=http \
CUHLUMN_INDEXNOW=off \
cargo run -p cuhlumn-server
```

Set `API_PROXY_TARGET` to send `/api` to a different server address.

## Shared TypeScript types

The `cuhlumn-api` crate generates the console wire types with `ts-rs`.
Run this command after a change to a wire type:

```sh
cargo test -p cuhlumn-api --features ts
```

The command writes one file for each type to `console/src/types/api/`.
Do not edit the generated files.

## Embedded console

The server embeds `console/dist` at compile time.
Debug builds read the files from disk, so a new console build does not need a server rebuild.
Release builds contain the files, so build the console before the release server.

## Documentation site

Run these commands in `docs/`:

```sh
uv sync --locked --group dev
uv run zensical serve    # local preview
uv run zensical build    # static site in docs/site
```
