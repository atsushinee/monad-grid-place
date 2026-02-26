# Use the official Rust image as a base
FROM rust:1-slim-buster as builder

# Create a new empty shell project
WORKDIR /usr/src/app
COPY . .

# Build the project
RUN cargo build --release

# Production image
FROM debian:buster-slim
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/indexer /usr/local/bin/indexer

# Set the entrypoint
CMD ["indexer"]
