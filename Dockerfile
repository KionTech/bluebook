# The API image: a cargo-chef staged build so dependency compilation is
# cached across code changes. Ships the server, the migrate tool, the
# demo seed and the three scheduled tasks as one-off commands.

FROM rust:1-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src src
COPY migrations migrations
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY src src
COPY migrations migrations
RUN cargo build --release --bin bluebook --bin migrate --bin seed_demo \
    --bin sync_standings --bin sync_affiliations --bin check_tokens

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/bluebook /app/target/release/migrate \
    /app/target/release/seed_demo /app/target/release/sync_standings \
    /app/target/release/sync_affiliations /app/target/release/check_tokens /usr/local/bin/
RUN useradd --system --create-home --uid 10001 app \
    && chown -R app:app /app
USER app
ENV BIND_ADDR=0.0.0.0:3000
EXPOSE 3000
CMD ["bluebook"]
