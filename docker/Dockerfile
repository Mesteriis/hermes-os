FROM rust:1.89-bookworm AS dev

ARG NODE_MAJOR=22

ENV DEBIAN_FRONTEND=noninteractive

RUN rustup component add rustfmt clippy

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        bash \
        ca-certificates \
        curl \
        git \
        gnupg \
        make \
        pkg-config \
        libssl-dev \
        protobuf-compiler \
    && install -d -m 0755 /etc/apt/keyrings \
    && curl -fsSL "https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key" \
        | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg \
    && echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_${NODE_MAJOR}.x nodistro main" \
        > /etc/apt/sources.list.d/nodesource.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends nodejs \
    && corepack enable \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

CMD ["bash"]

FROM dev AS backend-builder

COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
COPY crates ./crates

RUN cargo build \
    --manifest-path backend/Cargo.toml \
    --bin hermes-whatsapp-business-cloud-edge-proxy \
    --release

FROM debian:bookworm-slim AS whatsapp-business-cloud-edge-proxy

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=backend-builder \
    /workspace/target/release/hermes-whatsapp-business-cloud-edge-proxy \
    /usr/local/bin/hermes-whatsapp-business-cloud-edge-proxy

EXPOSE 8787

CMD ["hermes-whatsapp-business-cloud-edge-proxy"]
