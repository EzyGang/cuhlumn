# Builds the management console, then the server with the console embedded.

FROM node:22-trixie-slim AS console
WORKDIR /src
RUN corepack enable
COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
COPY console/package.json console/
RUN pnpm install --frozen-lockfile
COPY console console
RUN pnpm build

FROM rust:1-trixie AS server
WORKDIR /src
COPY Cargo.toml Cargo.lock clippy.toml ./
COPY crates crates
COPY --from=console /src/console/dist console/dist
RUN cargo build --release --locked -p cuhlumn-server

FROM debian:trixie-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates fonts-inter \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --home-dir /data cuhlumn \
    && mkdir /data \
    && chown cuhlumn /data
COPY --from=server /src/target/release/cuhlumn-server /usr/local/bin/cuhlumn-server
ENV CUHLUMN_DATA=/data PORT=8080
VOLUME /data
EXPOSE 8080
USER cuhlumn
CMD ["cuhlumn-server"]
