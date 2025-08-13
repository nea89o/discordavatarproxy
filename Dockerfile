FROM clux/muslrust:1.89.0 AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN rm rust-toolchain.toml
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Notice that we are specifying the --target flag!
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN rm rust-toolchain.toml
RUN cargo build --release --target x86_64-unknown-linux-musl --bin pfp

FROM docker.io/alpine:3 AS runtime
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/pfp /usr/local/bin/
CMD ["/usr/local/bin/pfp"]
