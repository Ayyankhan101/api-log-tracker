FROM rust:1.96 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src
COPY src/ src/
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/api_log_tracker /usr/local/bin/
RUN mkdir -p /app/logs
WORKDIR /app
EXPOSE 8080
CMD ["api_log_tracker", "daemon", "8080"]
