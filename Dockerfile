# syntax=docker/dockerfile:1
# Builds the management console, then the server with the console embedded.

FROM node:22-trixie-slim AS console
WORKDIR /src
RUN corepack enable
COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
COPY console/package.json console/
RUN --mount=type=cache,id=pnpm-store,target=/pnpm/store,sharing=locked \
    pnpm install --frozen-lockfile --store-dir /pnpm/store
COPY console console
RUN pnpm build

FROM rust:1-trixie AS server
ARG TARGETPLATFORM
WORKDIR /src
COPY Cargo.toml Cargo.lock clippy.toml ./
COPY crates crates
COPY --from=console /src/console/dist console/dist
# The target folder is a cache mount, so the binary is copied out of it.
RUN --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=cuhlumn-target-${TARGETPLATFORM},target=/src/target,sharing=locked \
    cargo build --release --locked -p cuhlumn-server \
    && cp target/release/cuhlumn-server /usr/local/bin/cuhlumn-server

FROM debian:trixie-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates fonts-inter \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --home-dir /data cuhlumn \
    && mkdir /data \
    && chown cuhlumn /data
COPY --from=server /usr/local/bin/cuhlumn-server /usr/local/bin/cuhlumn-server
ENV CUHLUMN_DATA=/data PORT=8080
VOLUME /data
EXPOSE 8080 8081
USER cuhlumn
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD ["cuhlumn-server", "healthcheck"]
CMD ["cuhlumn-server"]
