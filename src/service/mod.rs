use std::collections::HashSet;
use std::convert::Infallible;
use std::fs;
use std::path::Path;
use std::task::Poll;

use futures_util::stream::poll_fn;
use futures_util::Future;
use hyper::body::to_bytes;
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

    /// The data to inject at the end of the proxied response.
    pub inject: Vec<u8>,

    /// The marker position in the source data before which to inject data.
    pub marker: Vec<u8>,

    /// The paths for which to inject data.
    pub paths: HashSet<String>,
}

impl<'a, P> TryFrom<(&'a Configuration, P)> for Context
where
    P: AsRef<Path>,
{
    type Error = String;

    fn try_from(
        (source, path): (&'a Configuration, P),
    ) -> Result<Self, Self::Error> {
        let path = path
            .as_ref()
            .parent()
            .map(|directory| directory.join(&source.inject.source))
            .ok_or_else(|| {
                format!("invalid injected path: {:?}", path.as_ref())
            })?;
        Ok(Self {
            base_uri: source
                .base_uri
                .parse::<Uri>()
                .map_err(|e| e.to_string())?,
            client: Client::builder().build::<_, Body>(HttpsConnector::new()),
            inject: fs::read(&path)
                .map_err(|e| format!("failed to read {:?}: {}", path, e))?,
            marker: source.inject.marker.clone().into_bytes(),
            paths: source.inject.paths.clone(),
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
    // Remove the Accept-Encoding header to prevent compressed responses
    req.headers_mut().remove("accept-encoding");

    // Update the host header
    if let Some(host) = context
        .base_uri
        .host()
        .and_then(|host| HeaderValue::from_str(host).ok())
    {
        req.headers_mut().insert(HOST, host);
    }

    // Prepare the data to inject
    let inject = if context.paths.contains(req.uri().path()) {
        Some(context.inject.clone())
    } else {
        None
    };

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
            let builder = res
                .headers()
                .iter()
                .fold(Response::builder(), |acc, (k, v)| acc.header(k, v))
                .status(res.status());
            if let Some(inject) = inject {
                let mut body = Some(Box::pin(to_bytes(res.into_body())));
                builder.body(Body::wrap_stream::<_, _, Infallible>(poll_fn(
                    move |ctx| {
                        let data = match body
                            .as_mut()
                            .map(|body| body.as_mut().poll(ctx))
                        {
                            Some(Poll::Ready(d)) => Some(d
                                .map(|source| {
                                    insert_before(
                                        &source,
                                        &context.marker,
                                        &inject,
                                    )
                                })
                                .unwrap_or_else(|_| Vec::new())),
                            _ => None,
                        };
                        if let Some(data) = data {
                            // The remote request has completed
                            body = None;
                            Poll::Ready(Some(Ok(data)))
                        } else if body.is_none() {
                            // We have already sent the entire reply
                            Poll::Ready(None)
                        } else {
                            // We are still waiting for proxied server
                            Poll::Pending
                        }
                    },
                )))
            } else {
                builder.body(res.into_body())
            }
        })
}

/// Inserts `data` into `source` before `marker`.
///
/// If `marker` cannot be found in `source`, `source` is returned unmodified.
///
/// # Arguments
/// *  `source` - The source data.
/// *  `marker` - A marker sequence.
/// *  `data` - The data to insert.
fn insert_before(source: &[u8], marker: &[u8], data: &[u8]) -> Vec<u8> {
    source
        .windows(marker.len())
        .position(|window| window == marker)
        .map(|position| {
            let (head, tail) = source.split_at(position);
            let mut result = Vec::new();
            result.reserve(source.len() + data.len());
            result.extend_from_slice(head);
            result.extend_from_slice(data);
            result.extend_from_slice(tail);
            result
        })
        .unwrap_or(source.into())
}
