use hyper::{Body, Request, Response};
use std::convert::Infallible;

use crate::configuration::Configuration;

/// The context in which the service operates.
#[derive(Clone)]
pub struct Context {
    /// The base URI to proxy.
    pub base_uri: Uri,

    /// Our Hyper client.
    pub client: Client<HttpsConnector<HttpConnector>>,
}

impl<'a> TryFrom<&'a Configuration> for Context {
    type Error = String;

    fn try_from(source: &'a Configuration) -> Result<Self, Self::Error> {
        Ok(Self {
            base_uri: source
                .base_uri
                .parse::<Uri>()
                .map_err(|e| e.to_string())?,
            client: Client::builder().build::<_, Body>(HttpsConnector::new()),
        })
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
