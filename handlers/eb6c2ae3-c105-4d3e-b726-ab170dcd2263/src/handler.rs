//! Handler for this endpoint
use edge_hive_sdk::prelude::*;

/// Handle incoming requests
pub fn handle(req: Request) -> Response {
    Response::ok(json!({
        "message": "Hello, World!",
        "path": req.path,
        "method": req.method
    }))
}
