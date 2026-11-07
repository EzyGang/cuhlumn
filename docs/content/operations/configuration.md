# Configuration

The server reads a small number of settings from environment variables or command-line flags.
You change the other settings in the management console.
Run `cuhlumn-server --help` to see the flags.

| Variable | Flag | Default | Meaning |
| --- | --- | --- | --- |
| `PORT` | `--port` | `8080` | The port on `0.0.0.0` |
| `CUHLUMN_DATA` | `--data` | `./data` | The folder for `cuhlumn.db`, `media/`, and `fonts/` |
| `CUHLUMN_TOKEN` | `--token` | None | Optional override of the admin token, at least 32 bytes |
| `CUHLUMN_CONSOLE_HOST` | `--console-host` | None | Optional override of the console domain |
| `CUHLUMN_SCHEME` | `--scheme` | `https` | Optional override of the scheme, `http` or `https` |
| `RUST_LOG` | None | `info` | The log filter, for example `cuhlumn_server=debug` |

The Docker image sets `CUHLUMN_DATA=/data`.

## Server settings and overrides

The server keeps the admin token, the console domain, and the scheme in the database.
You change them on the **Server settings** page of the console, or with the [API](../reference/api.md).

Three environment variables override these settings:

- When an override variable is set, its value wins over the database value.
- The console then shows the field as locked, with the text "Set by" and the variable name.
- A request that changes a locked field to a different value returns `409` with the code `locked`.
- When the variable is not set, the server uses the database value.

IndexNow is a setting of each site, and fonts for share images are uploaded on the **Fonts** page of the console.

## Admin token

- Without a token in the environment or the database, the server starts in setup mode. See [First launch](../getting-started/installation.md#first-launch).
- In setup mode, the server serves the blogs, and admin routes return `503` with the code `needs_setup`.
- A token shorter than 32 bytes stops the server at startup.
- The database stores only the SHA-256 digest of the token.
- A new token ends all console sessions.

Generate a token with `openssl rand -hex 32`.

## Console domain

When a console domain is set, only that domain serves the console.
When no console domain is set, every domain that is not a blog domain or an alias serves the console.
This includes the IP address of the server.
A blog domain or alias cannot be the same as the console domain.

## Scheme

The scheme setting does not turn on TLS.
It tells the server which scheme the readers use, so the addresses in pages, feeds, and API answers are correct.
It also sets the `Secure` flag of the session cookie.
Use `http` only for local work.
A change of the scheme clears the page cache.

## Logs

The server writes logs to standard output with `tracing`.
An invalid `RUST_LOG` value stops the server at startup.

## Startup and shutdown

At startup the server creates the data folder, opens the database, applies new migrations, and deletes expired sessions.
On Ctrl-C, or SIGTERM on Unix, the server stops accepting connections and finishes the requests in progress.

## Local development

```sh
CUHLUMN_TOKEN=dev-token-0123456789abcdef0123456789 \
CUHLUMN_SCHEME=http \
cargo run -p cuhlumn-server
```

Use domains that end in `.localhost`, such as `notes.localhost`, for local blogs.
Browsers send these domains to the local computer.
Turn off IndexNow in the settings of each local site.
