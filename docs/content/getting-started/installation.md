# Installation

You can run Cuhlumn with Docker or build it from source.

## Docker

The `Dockerfile` builds the console, then builds the server with the console embedded.
The final image is based on `debian:trixie-slim` and contains the Inter font for share images.

```sh
docker build -t cuhlumn .
docker run -d --name cuhlumn -p 8080:8080 -v cuhlumn:/data \
  -e CUHLUMN_TOKEN="$(openssl rand -hex 32)" \
  -e CUHLUMN_CONSOLE_HOST=admin.example.com \
  cuhlumn
```

| Image property | Value |
| --- | --- |
| Binary | `/usr/local/bin/cuhlumn-server` |
| User | `cuhlumn`, user id `10001` |
| Data volume | `/data` |
| Port | `8080` |
| Environment | `CUHLUMN_DATA=/data`, `PORT=8080` |

The image has no `HEALTHCHECK`.
Use `GET /api/health` for health checks.

## From source

Install stable Rust, Node.js 22.12 or later, and pnpm 11.

```sh
pnpm install
pnpm build
cargo build --release -p cuhlumn-server
CUHLUMN_TOKEN=change-me-to-a-long-random-value ./target/release/cuhlumn-server
```

The release build embeds the console from `console/dist`.
Build the console before the server.

## Domains and TLS

1. Point every blog domain, every alias, and the console domain at the server.
2. Put a TLS proxy in front of the server.
3. Make sure that the proxy passes the original `Host` header.

The server listens on `0.0.0.0` with plain HTTP.
`CUHLUMN_SCHEME` sets the scheme in the addresses that the server writes, which is `https` by default.

We recommend [Cloudflare](../operations/cloudflare.md) as the proxy in front of the blog domains.
Cloudflare caches the pages near the readers, so the server renders much less.

## Check the installation

```sh
curl http://127.0.0.1:8080/api/health
```

The answer is `{"ok":true,"version":"0.1.0"}`.

See [Configuration](../operations/configuration.md) for all settings.
