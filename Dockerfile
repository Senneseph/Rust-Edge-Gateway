# Edge Hive Platform - Development & Runtime Container
FROM rust:1.83-slim-bookworm

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    sqlite3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace files
COPY . .

# Build the project (will be done on first run if not cached)
# RUN cargo build --release

# Expose ports
# 8080 - Main gateway
# 8081 - Admin interface
EXPOSE 8080 8081

# Default command
CMD ["cargo", "run", "--bin", "edge-hive"]

