# -----------------------------
# Build battleship base image
# -----------------------------

FROM rust as builder
WORKDIR /usr/src

# Prepare for static linking with musl
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl

# Download and compile Rust dependencies in an empty project and cache as a separate Docker layer
RUN USER=root cargo new --bin battleship-temp
WORKDIR /usr/src/battleship-temp
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release --target x86_64-unknown-linux-musl
# remove src form empty project
RUN rm -r src

# Copy actual source files and Build the app binary
COPY src ./src
# due to cargo bug https://github.com/rust-lang/rust/issues/25289
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev libxcb-composite0-dev
# remove previous deps
RUN rm ./target/x86_64-unknown-linux-musl/release/deps/battleship*
RUN cargo build --release --target x86_64-unknown-linux-musl

# -----------------------------
# build final battleship image
# -----------------------------

FROM alpine:latest

# Copy the compiled binary from the builder container
COPY --from=builder /usr/src/battleship-temp/target/x86_64-unknown-linux-musl/release/battleship /usr/local/bin

ENTRYPOINT [ "/usr/local/bin/battleship" ]
