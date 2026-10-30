FROM node:24-bookworm

ENV DEBIAN_FRONTEND=noninteractive \
    CARGO_HOME=/opt/cargo \
    RUSTUP_HOME=/opt/rustup \
    RUST_BACKTRACE=1 \
    RUSTFLAGS="-C force-frame-pointers=yes"
ENV PATH="/opt/cargo/bin:${PATH}"

RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential clang cmake curl git libssl-dev ninja-build \
      pkg-config python3 gdb \
    && rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh \
    && sh /tmp/rustup-init.sh -y --profile minimal --default-toolchain none \
    && rm /tmp/rustup-init.sh

COPY . /src
WORKDIR /src

# Resolve the tool versions from the checkout being scanned.
RUN rustup show active-toolchain \
    && rustup component add rust-src \
    && rustup target add wasm32-unknown-unknown \
    && npm install --global "$(node -p 'require("./package.json").packageManager')"
RUN pnpm install --frozen-lockfile
RUN ./scripts/clone_fixtures.sh
RUN cargo fetch --locked \
    && cargo build --workspace --locked \
    && cargo test --workspace --locked --no-run \
    && pnpm build

# Compile and run focused tests without allowing an implicit network fetch.
ENV CARGO_NET_OFFLINE=true
RUN cargo test --offline --locked -p swc_next_allocator -p swc_next_simd \
      -p swc_next_ecma_ast -p swc_next_ecma_parser \
    && pnpm test
