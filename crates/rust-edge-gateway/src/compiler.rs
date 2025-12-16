//! Handler compilation service
//!
//! Compiles uploaded Rust source files into dynamic library handlers (v2 architecture).

use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;
use tokio::task;

use crate::config::AppConfig;

/// Template for handler Cargo.toml (v2 - dynamic library)
const CARGO_TOML_TEMPLATE: &str = r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[workspace]

[lib]
crate-type = ["cdylib"]

[dependencies]
rust-edge-gateway-sdk = { path = "{sdk_path}" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
"#;

/// Template for handler lib.rs wrapper (v2 - dynamic library entry point)
///
/// The user's handler.rs should define a function like:
/// ```ignore
/// pub fn handle(ctx: &impl HandlerContext, req: Request) -> Response {
///     Response::ok(json!({"message": "Hello!"}))
/// }
/// ```
const LIB_RS_TEMPLATE: &str = r#"//! Auto-generated handler wrapper (v2 dynamic library)
use rust_edge_gateway_sdk::prelude::*;
use std::pin::Pin;
use std::future::Future;

mod handler;

/// Entry point called by the gateway to handle requests.
/// This is the v2 dynamic library interface.
#[no_mangle]
pub extern "C" fn handler_entry<Ctx: HandlerContext>(
    ctx: &Ctx,
    req: Request,
) -> Pin<Box<dyn Future<Output = Response> + Send + 'static>> {
    // Clone what we need to make the future 'static
    let req = req;
    // We need to use unsafe to transmute the lifetime since the context
    // is guaranteed to live for the duration of the handler call
    let ctx_ptr = ctx as *const Ctx;
    Box::pin(async move {
        let ctx = unsafe { &*ctx_ptr };
        handler::handle(ctx, req)
    })
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
    // Assuming handlers_dir is at ./handlers and SDK is at ./crates/rust-edge-gateway-sdk
    let sdk_path = "../../crates/rust-edge-gateway-sdk";

    // Package name must start with a letter, so prefix with "handler_"
    let package_name = format!("handler_{}", id.replace('-', "_"));

    // Write Cargo.toml
    let cargo_toml = CARGO_TOML_TEMPLATE
        .replace("{name}", &package_name)
        .replace("{sdk_path}", sdk_path);
    std::fs::write(handler_dir.join("Cargo.toml"), cargo_toml)?;

    // Write lib.rs wrapper (v2 dynamic library entry point)
    std::fs::write(src_dir.join("lib.rs"), LIB_RS_TEMPLATE)?;

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

    // Determine the library filename based on platform
    let lib_filename = format_library_name(&package_name);

    // The library is built in target/release/
    let lib_in_target = handler_dir
        .join("target")
        .join("release")
        .join(&lib_filename);

    if !lib_in_target.exists() {
        return Err(anyhow!("Library not found after compilation: {:?}", lib_in_target));
    }

    // Copy the library to the handler directory root for the registry to find
    // The registry expects: handlers/{id}/libhandler_{id}.so
    let lib_dest = handler_dir.join(&lib_filename);
    std::fs::copy(&lib_in_target, &lib_dest)?;

    tracing::info!("Handler compiled: {:?}", lib_dest);

    Ok(lib_dest.to_string_lossy().to_string())
}

/// Format the library filename for the current platform
#[cfg(target_os = "windows")]
fn format_library_name(package_name: &str) -> String {
    format!("{}.dll", package_name)
}

#[cfg(target_os = "linux")]
fn format_library_name(package_name: &str) -> String {
    format!("lib{}.so", package_name)
}

#[cfg(target_os = "macos")]
fn format_library_name(package_name: &str) -> String {
    format!("lib{}.dylib", package_name)
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn format_library_name(package_name: &str) -> String {
    format!("lib{}.so", package_name)
}

