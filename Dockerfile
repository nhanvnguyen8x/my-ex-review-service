# Build
FROM rust:1-bookworm AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
COPY migrations ./migrations

# Build release (no compile-time DB check)
RUN cargo build --release

# Run
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/my-ex-review-service /usr/local/bin/
EXPOSE 3005
ENV PORT=3005
CMD ["my-ex-review-service"]
