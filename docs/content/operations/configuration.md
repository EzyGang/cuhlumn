# Configuration

The server reads its settings from environment variables or command-line flags.
Run `cuhlumn-server --help` to see them.

| Variable | Flag | Default | Meaning |
| --- | --- | --- | --- |
| `PORT` | `--port` | `8080` | The port on `0.0.0.0` |
| `CUHLUMN_DATA` | `--data` | `./data` | The folder for `cuhlumn.db` and `media/` |
| `CUHLUMN_TOKEN` | `--token` | None | The admin token, at least 32 bytes |
| `CUHLUMN_SCHEME` | `--scheme` | `https` | `http` or `https`, for absolute addresses and the cookie `Secure` flag |
| `CUHLUMN_CONSOLE_HOST` | `--console-host` | None | The domain that serves the management console |
| `CUHLUMN_INDEXNOW` | `--indexnow` | `on` | `on` or `off` |
| `CUHLUMN_FONTS` | `--fonts` | None | An extra font folder for share images |
| `RUST_LOG` | None | `info` | The log filter, for example `cuhlumn_server=debug` |

## Admin token

- Without `CUHLUMN_TOKEN`, the server serves the blogs, and the API and the console return `503`.
- A token shorter than 32 bytes stops the server at startup.
- An empty `CUHLUMN_TOKEN` counts as a short token. Remove the variable to turn the API off.

Generate a token with `openssl rand -hex 32`.

## Console domain

Without `CUHLUMN_CONSOLE_HOST`, the console is off and the session routes return `404`.
The API still works with the bearer token.

## Scheme

`CUHLUMN_SCHEME` does not turn on TLS.
It tells the server which scheme the readers use, so the addresses in pages, feeds, and API answers are correct.
Use `http` only for local work.

## Logs

The server writes logs to standard output with `tracing`.
An invalid `RUST_LOG` value stops the server at startup.

## Startup and shutdown

At startup the server creates the data folder, opens the database, applies new migrations, and deletes expired sessions.
On Ctrl-C, or SIGTERM on Unix, the server stops accepting connections and finishes the requests in progress.

## Local development

```sh
CUHLUMN_TOKEN=dev-token-0123456789abcdef0123456789 \
CUHLUMN_CONSOLE_HOST=localhost \
CUHLUMN_SCHEME=http \
CUHLUMN_INDEXNOW=off \
cargo run -p cuhlumn-server
```

Use domains that end in `.localhost`, such as `notes.localhost`, for local blogs.
Browsers send these domains to the local computer.
