use rust_edge_gateway_sdk::prelude::*;

/// Hello World handler
/// Returns a simple JSON response with "Hello, World!" message
#[handler]
pub async fn handle(_ctx: &Context, _req: Request) -> Response {
    Response::ok(json!({
        "message": "Hello, World!"
    }))
}
