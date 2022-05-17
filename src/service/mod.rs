use hyper::{Body, Request, Response};
use std::convert::Infallible;

/// Handles a request to this service.
///
/// # Arguments
/// *  `_req` - The request to handle.
pub async fn handle(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("rust-roller".into()))
}
