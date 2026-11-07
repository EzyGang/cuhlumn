# Installation

You can run Cuhlumn with Docker Compose, with Docker only, or build it from source.
Docker Compose is the recommended method.

## Docker Compose

The repository contains `docker-compose.yml` and `.env.example`.
Compose starts the image `ghcr.io/ezygang/cuhlumn:latest` and keeps the data in the volume `cuhlumn-data`.

1. Download `docker-compose.yml` and `.env.example` into one folder.
2. Copy `.env.example` to `.env`.
3. Set the variables in `.env`.
4. Start the server with `docker compose up -d`.

```sh
cp .env.example .env
openssl rand -hex 32    # paste the result into CUHLUMN_TOKEN in .env
docker compose up -d
```

The server port is published only on `127.0.0.1:8080` of the host.
Put a TLS proxy or a Cloudflare tunnel in front of it.

| `.env` variable | Meaning |
| --- | --- |
| `CUHLUMN_IMAGE` | The image, `ghcr.io/ezygang/cuhlumn:latest` by default |
| `CUHLUMN_PORT` | The host port on `127.0.0.1`, `8080` by default |
| `CUHLUMN_TOKEN` | Optional admin token, see [First launch](#first-launch) |
| `CUHLUMN_CONSOLE_HOST` | Optional console domain |
| `CUHLUMN_SCHEME` | Optional scheme, `https` by default |
| `RUST_LOG` | The log filter |
| `CLOUDFLARE_TUNNEL_TOKEN` | The tunnel token, for the `tunnel` profile only |

The `tunnel` profile also starts `cloudflared`.
Start it with `docker compose --profile tunnel up -d`.
See [Cloudflare](../operations/cloudflare.md#cloudflare-tunnel-with-compose).

To use a fixed version, set `CUHLUMN_IMAGE=ghcr.io/ezygang/cuhlumn:1.0.0` in `.env`.
The release workflow also publishes the tags `latest`, `MAJOR`, and `MAJOR.MINOR`.

### Container settings

Compose runs the container with these protections:

- A read-only root file system with a `tmpfs` on `/tmp`
- No Linux capabilities and `no-new-privileges`
- `restart: unless-stopped`
- A health check with the command `cuhlumn-server healthcheck`

## Docker

You can start the image without Compose.

```sh
docker run -d --name cuhlumn -p 127.0.0.1:8080:8080 -v cuhlumn-data:/data \
  ghcr.io/ezygang/cuhlumn:latest
```

To build the image yourself, run `docker build -t cuhlumn .` in the repository.
The `Dockerfile` builds the console, then the server with the console embedded.

| Image property | Value |
| --- | --- |
| Base | `debian:trixie-slim` with the Inter font |
| Platforms | `linux/amd64`, `linux/arm64` |
| Binary | `/usr/local/bin/cuhlumn-server` |
| User | `cuhlumn`, user id `10001` |
| Data volume | `/data` |
| Port | `8080` |
| Environment | `CUHLUMN_DATA=/data`, `PORT=8080` |
| Health check | `cuhlumn-server healthcheck` |

## Health check command

`cuhlumn-server healthcheck` sends `GET /api/health` to `127.0.0.1` on `PORT`, with a timeout of 3 seconds.
It exits with `0` when the answer is `200` with `"ok": true`.
Otherwise it exits with `1`.
The command does not open the data folder.

## From source

Install stable Rust, Node.js 22.12 or later, and pnpm 11.

```sh
pnpm install
pnpm build
cargo build --release -p cuhlumn-server
./target/release/cuhlumn-server
```

The release build embeds the console from `console/dist`.
Build the console before the server.

## First launch

Without an admin token, the server starts in setup mode:

- The health answer contains `"needs_setup": true`.
- Admin routes return `503` with the code `needs_setup`.
- The console shows a **Setup** page.

On the Setup page, enter a token with at least 32 characters, or select **Generate**.
Keep the token in a safe place, then submit the form.
The server stores the SHA-256 digest of the token and logs you in.

!!! warning "Anyone who reaches the server first can claim it"
    In setup mode, the first visitor of the console sets the admin token.
    Before the setup, the console answers on every domain that is not a blog domain, also on the IP address of the server.
    To prevent this, do one of these steps:

    - Set `CUHLUMN_TOKEN` before the first start.
    - Keep the port private until you complete the setup, for example with an SSH tunnel.

When `CUHLUMN_TOKEN` is set, the server does not use setup mode.
See [Configuration](../operations/configuration.md).

## Domains and TLS

1. Point every blog domain, every alias, and the console domain at the server.
2. Put a TLS proxy in front of the server.
3. Make sure that the proxy passes the original `Host` header.
4. Save the console domain on the **Server settings** page of the console.

The server listens on `0.0.0.0` with plain HTTP.
The scheme setting sets the scheme in the addresses that the server writes, which is `https` by default.

We recommend [Cloudflare](../operations/cloudflare.md) as the proxy in front of the blog domains.
Cloudflare caches the pages near the readers, so the server renders much less.

## Check the installation

```sh
curl http://127.0.0.1:8080/api/health
```

The answer has this form:

```json
{"ok": true, "version": "0.1.0", "needs_setup": false}
```

See [Configuration](../operations/configuration.md) for all settings.
