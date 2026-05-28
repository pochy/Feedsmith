FROM rust:1.78-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY askama.toml ./
COPY migrations ./migrations
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/* \
  && mkdir -p /data
COPY --from=builder /app/target/release/feedsmith /usr/local/bin/feedsmith
COPY static ./static
ENV APP_HOST=0.0.0.0 \
    APP_PORT=3000 \
    DATABASE_URL=sqlite:///data/feedsmith.db
EXPOSE 3000
CMD ["feedsmith"]
