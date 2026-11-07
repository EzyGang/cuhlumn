# Configuration

The server reads a small number of settings from environment variables or command-line flags.
You change the other settings in the management console.
Run `cuhlumn-server --help` to see the flags.

| Variable | Flag | Default | Meaning |
| --- | --- | --- | --- |
| `PORT` | `--port` | `8080` | The port on `0.0.0.0` for the blogs |
| `CUHLUMN_ADMIN_ADDR` | `--admin-addr` | None | Optional address of the admin listener, such as `0.0.0.0:8081` |
| `CUHLUMN_DATA` | `--data` | `./data` | The folder for `cuhlumn.db`, `media/`, and `fonts/` |
| `CUHLUMN_TOKEN` | `--token` | None | Optional override of the admin token, at least 32 bytes |
| `CUHLUMN_CONSOLE_HOST` | `--console-host` | None | Optional override of the console domain |
| `CUHLUMN_SCHEME` | `--scheme` | `https` | Optional override of the scheme, `http` or `https` |
| `RUST_LOG` | None | `info` | The log filter, for example `cuhlumn_server=debug` |

The Docker image sets `CUHLUMN_DATA=/data`.
An empty variable counts as not set.

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

## Admin listener

`CUHLUMN_ADMIN_ADDR` starts a second listener for the console and the admin API.
The value is a socket address with a port, such as `0.0.0.0:8081` or `100.92.3.4:8081`.

| Listener | Serves |
| --- | --- |
| `PORT` | The blogs and `GET /api/health`. Every other `/api` path returns `404` with the code `not_found`. |
| `CUHLUMN_ADMIN_ADDR` | The full `/api` and the console for every other path, on every host name |

- The public port never serves the console, also before the setup.
- The console domain setting has no effect. The console shows the field as locked, and `GET /api/settings` reports `admin_listener: true`.
- A blog domain or alias can be any name, because no domain serves the console.
- The scheme setting still controls the addresses of the blogs.
- The health check of the Docker image uses `PORT`, so it works with both setups.

Use the admin listener on a private network, such as NetBird or another WireGuard network.
See [Private admin port](../getting-started/installation.md#private-admin-port).
Do not expose the admin port to the public internet.

### HTTP and HTTPS on the admin port

The admin port works with HTTPS and without HTTPS.

- HTTPS is supported and recommended when the network does not encrypt traffic. Put a TLS proxy in front of the admin port that sets `X-Forwarded-Proto: https`. The session cookie then gets `Secure`.
- Plain HTTP also works. This is acceptable on a private network that encrypts traffic itself, such as NetBird or another WireGuard network.
- On plain HTTP, browsers block the clipboard API. The token **Copy** buttons then select the token, and the media library shows the address in a message, so you can copy it by hand.
- The session cookie gets `Secure` only when the request has `X-Forwarded-Proto: https`.

## Scheme

The scheme setting does not turn on TLS.
It tells the server which scheme the readers use, so the addresses in pages, feeds, and API answers are correct.
Without an admin listener, it also sets the `Secure` flag of the session cookie.
Use `http` only for local work.
A change of the scheme clears the page cache.

## Logs

The server writes logs to standard output with `tracing`.
An invalid `RUST_LOG` value stops the server at startup.

## Startup and shutdown

At startup the server creates the data folder, opens the database, applies new migrations, and deletes expired sessions.
On Ctrl-C, or SIGTERM on Unix, the server stops accepting connections and finishes the requests in progress.
The startup log shows the address of each listener.
With an admin listener, one shutdown signal stops both listeners.
When the server cannot listen on an address, the error names the address and the variable that sets it.

## Local development

```sh
CUHLUMN_TOKEN=dev-token-0123456789abcdef0123456789 \
CUHLUMN_SCHEME=http \
cargo run -p cuhlumn-server
```

Use domains that end in `.localhost`, such as `notes.localhost`, for local blogs.
Browsers send these domains to the local computer.
Turn off IndexNow in the settings of each local site.
