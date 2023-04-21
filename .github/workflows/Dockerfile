ARG NODE_VERSION=20
FROM node:${NODE_VERSION}-alpine
ARG RUSTFLAGS="-C target-feature=-crt-static"
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

WORKDIR /app
COPY . .
RUN apk add --update --virtual build-deps g++ make cmake zlib curl git python3 \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile=minimal -y \
    && yarn install --production=false --ignore-scripts \
    && yarn exec -- cargo-cp-artifact -nc native/index.node -- cargo build --release --manifest-path=native/Cargo.toml --message-format=json-render-diagnostics \
    && rm -rf ./native/target \
    && apk del build-deps \
    && echo "SHA-512 of native/index.node: $(sha512sum native/index.node | awk '{ print $1 }')"
