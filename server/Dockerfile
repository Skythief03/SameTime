# ---- Build Stage ----
FROM rust:1.78-bookworm AS builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Build actual binary
COPY src/ src/
RUN touch src/main.rs && cargo build --release

# ---- Runtime Stage ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/sametime-server /app/sametime-server

# Create data directories
RUN mkdir -p /app/data/uploads

# Default environment
ENV SAMETIME_HOST=0.0.0.0
ENV SAMETIME_PORT=8080
ENV DATABASE_URL=sqlite:./data/sametime.db?mode=rwc
ENV UPLOAD_DIR=./data/uploads
ENV JWT_SECRET=change-me-in-production
ENV MAX_FILE_SIZE=10737418240

EXPOSE 8080

VOLUME ["/app/data"]

CMD ["./sametime-server"]