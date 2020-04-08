ARG NODE_VERSION=13
FROM node:${NODE_VERSION}-alpine
ARG RUSTFLAGS="-C target-feature=-crt-static"
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

WORKDIR /app
COPY . .
RUN apk add --update --virtual build-deps g++ make zlib curl git python3 python2 \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile=minimal -y \
    && yarn --ignore-scripts \
    && yarn exec -- neon build --release \
    && rm -rf ./native/target \
    && apk del build-deps \
    && echo "SHA256 of native/index.node: $(sha256sum native/index.node | awk '{ print $1 }')"
