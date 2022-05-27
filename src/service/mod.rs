use hyper::client::HttpConnector;
use hyper::header::{HeaderValue, HOST};
use hyper::http::Error;
use hyper::{Body, Client, Method, Request, Response, StatusCode, Uri};
use hyper_tls::HttpsConnector;

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
/// *  `context` - The application context.
/// *  `req` - The request to handle.
pub async fn handle(
    context: Context,
    req: Request<Body>,
) -> Result<Response<Body>, Error> {
    match req.method() {
        &Method::GET => handle_get(context, req).await,
        _ => Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Body::empty()),
    }
}

/// Proxies `GET` requests.
///
/// # Arguments
/// *  `context` - The application context.
/// *  `req` - The request to handle.
async fn handle_get(
    context: Context,
    mut req: Request<Body>,
) -> Result<Response<Body>, Error> {
    // Update the host header
    if let Some(host) = context
        .base_uri
        .host()
        .and_then(|host| HeaderValue::from_str(host).ok())
    {
        req.headers_mut().insert(HOST, host);
    }

    // Update the request URI
    let remote_uri = req
        .uri()
        .path_and_query()
        .and_then(|pq| {
            (context.base_uri.to_string() + pq.as_str()).parse().ok()
        })
        .unwrap_or_else(|| context.base_uri.clone());
    *req.uri_mut() = remote_uri;

    context
        .client
        .request(req)
        .await
        .or_else(|_| {
            Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::empty())
        })
        .and_then(|res| {
            res.headers()
                .iter()
                .fold(Response::builder(), |acc, (k, v)| acc.header(k, v))
                .status(res.status())
                .body(res.into_body())
        })
}
