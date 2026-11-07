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

## CI commands

The `CI` workflow in `.github/workflows/ci.yml` runs these commands from the repository root.
Run them before you push.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p cuhlumn-api --features ts --test ts_export --locked   # then commit the changes in console/src/types/api
cargo test --workspace --all-features --locked
pnpm install --frozen-lockfile
pnpm validate
pnpm format:check
pnpm test
(cd docs && uv sync --locked --group dev && uv run zensical build)
```

The integration tests send requests through the router against a temporary data folder.
They do not need a network connection.

Build the release server with `cargo build --release -p cuhlumn-server`.

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
CUHLUMN_SCHEME=http \
cargo run -p cuhlumn-server
```

Without a saved console domain, `localhost` serves the console.
Set `API_PROXY_TARGET` to send `/api` to a different server address.

## Shared TypeScript types

The `cuhlumn-api` crate generates the console wire types with `ts-rs`.
Run this command after a change to a wire type:

```sh
cargo test -p cuhlumn-api --features ts --test ts_export
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

## Release

The `Release` workflow in `.github/workflows/release.yml` publishes a version.

1. Open **Actions** > **Release** on GitHub.
2. Select the `main` branch.
3. Enter the version without the `v` prefix, for example `1.4.0`.
4. Select **Run workflow**.

The workflow runs these jobs in sequence:

| Job | Action |
| --- | --- |
| `validate` | Checks the branch, the semver form, a free tag, and a version higher than the current version |
| `ci` | Runs the `CI` workflow |
| `prepare` | Writes the version to `Cargo.toml`, `console/package.json`, `Cargo.lock`, and the health examples in the docs, then commits, tags, and pushes |
| `build` | Builds the image for `linux/amd64` and `linux/arm64` and pushes it by digest |
| `smoke` | Starts the `amd64` image with Compose, checks the health answer, creates a site, and publishes a post |
| `publish` | Pushes the multi-platform tags and creates the GitHub release |

A version with a pre-release part, such as `1.4.0-rc.1`, gets only its own tag.
Other versions also get the tags `MAJOR.MINOR`, `MAJOR`, and `latest`.

The `prepare` job pushes to `main` with the deploy key in the secret `RELEASE_DEPLOY_KEY`.

!!! note "Make the package public"
    GitHub creates the `ghcr.io/ezygang/cuhlumn` package as private at the first release.
    After the first release, change the package visibility to public in the package settings.
    Otherwise, `docker compose pull` fails without a login.
