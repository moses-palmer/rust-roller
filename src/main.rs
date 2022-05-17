use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

mod service;

#[tokio::main]
async fn main() {
    let address = SocketAddr::from(([127, 0, 0, 1], 8080));
    let service = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(service::handle))
    });
    if let Err(e) = Server::bind(&address).serve(service).await {
        eprintln!("server error: {}", e);
    }
}
