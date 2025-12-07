//! Handler compilation service
//!
//! Compiles uploaded Rust source files into handler binaries.

use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;
use tokio::task;

use crate::config::AppConfig;

/// Template for handler Cargo.toml
const CARGO_TOML_TEMPLATE: &str = r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
edge-hive-sdk = { path = "{sdk_path}" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
"#;

/// Template for handler main.rs wrapper
const MAIN_RS_TEMPLATE: &str = r#"//! Auto-generated handler wrapper
use edge_hive_sdk::prelude::*;

mod handler;

fn main() {
    loop {
        match edge_hive_sdk::ipc::read_request() {
            Ok(req) => {
                let response = handler::handle(req);
                if let Err(e) = edge_hive_sdk::ipc::send_response(response) {
                    eprintln!("Failed to send response: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to read request: {}", e);
                break;
            }
        }
    }
}
"#;

/// Compile a handler from source code
pub async fn compile_handler(config: &AppConfig, id: &str, code: &str) -> Result<String> {
    let handlers_dir = config.handlers_dir.clone();
    let id = id.to_string();
    let code = code.to_string();
    
    // Run compilation in a blocking task
    task::spawn_blocking(move || {
        compile_handler_sync(&handlers_dir, &id, &code)
    }).await?
}

fn compile_handler_sync(handlers_dir: &PathBuf, id: &str, code: &str) -> Result<String> {
    // Create handler directory structure
    let handler_dir = handlers_dir.join(id);
    let src_dir = handler_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    // Calculate relative path to SDK
    // Assuming handlers_dir is at ./handlers and SDK is at ./crates/edge-hive-sdk
    let sdk_path = "../../crates/edge-hive-sdk";

    // Package name must start with a letter, so prefix with "handler_"
    let package_name = format!("handler_{}", id.replace('-', "_"));

    // Write Cargo.toml
    let cargo_toml = CARGO_TOML_TEMPLATE
        .replace("{name}", &package_name)
        .replace("{sdk_path}", sdk_path);
    std::fs::write(handler_dir.join("Cargo.toml"), cargo_toml)?;
    
    // Write main.rs wrapper
    std::fs::write(src_dir.join("main.rs"), MAIN_RS_TEMPLATE)?;
    
    // Write user's handler code
    std::fs::write(src_dir.join("handler.rs"), code)?;
    
    // Compile with cargo
    tracing::info!("Compiling handler {} in {:?}", id, handler_dir);

    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&handler_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Compilation failed:\n{}", stderr));
    }

    // Return path to the compiled binary (binary name matches package name)
    let binary_path = handler_dir
        .join("target")
        .join("release")
        .join(&package_name);

    if !binary_path.exists() {
        return Err(anyhow!("Binary not found after compilation"));
    }

    Ok(binary_path.to_string_lossy().to_string())
}

