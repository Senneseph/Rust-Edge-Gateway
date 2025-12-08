#!/bin/bash
set -e

echo "=== Initializing Rust Edge Gateway Rust Workspace ==="

# Create workspace Cargo.toml
cat > Cargo.toml << 'EOF'
[workspace]
resolver = "2"
members = [
    "crates/rust-edge-gateway-sdk",
    "crates/rust-edge-gateway",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
authors = ["Rust Edge Gateway Team"]

[workspace.dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }

# Web framework
axum = { version = "0.8", features = ["macros"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["fs", "cors", "trace"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Database
rusqlite = { version = "0.32", features = ["bundled"] }

# Utilities
thiserror = "2"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# SDK specific
bytes = "1"
EOF

# Create directory structure
mkdir -p crates/rust-edge-gateway-sdk/src
mkdir -p crates/rust-edge-gateway/src
mkdir -p handlers
mkdir -p static/admin
mkdir -p data

echo "=== Creating rust-edge-gateway-sdk crate ==="
cat > crates/rust-edge-gateway-sdk/Cargo.toml << 'EOF'
[package]
name = "rust-edge-gateway-sdk"
version.workspace = true
edition = "2021"
license.workspace = true
description = "SDK for writing Rust Edge Gateway handlers"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
bytes = { workspace = true }

[dev-dependencies]
EOF

echo "=== Creating rust-edge-gatewaye-gateway crate ==="
cat > crates/rust-edge-gateway/Cargo.toml << 'EOF'
[package]
name = "rust-edge-gateway"
version.workspace = true
edition = "2021"
license.workspace = true
description = "Rust Edge Gateway and Worker Supervisor"

[[bin]]
name = "rust-edge-gateway"
path = "src/main.rs"

[dependencies]
rust-edge-gateway-sdk = { path = "../rust-edge-gateway-sdk" }
tokio = { workspace = true }
axum = { workspace = true }
tower = { workspace = true }
tower-http = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
rusqlite = { workspace = true }
thiserror = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
EOF

echo "=== Workspace initialized ==="
echo "Directory structure:"
find . -type f -name "Cargo.toml" | head -20

