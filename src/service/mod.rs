use hyper::{Body, Request, Response};
use std::convert::Infallible;

use crate::configuration::Configuration;

/// The context in which the service operates.
#[derive(Clone)]
pub struct Context {}

impl<'a> TryFrom<&'a Configuration> for Context {
    type Error = String;

    fn try_from(_source: &'a Configuration) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}

/// Handles a request to this service.
///
/// # Arguments
/// *  `_context` - The application context.
/// *  `_req` - The request to handle.
pub async fn handle(
    _context: Context,
    _req: Request<Body>,
) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("rust-roller".into()))
}
