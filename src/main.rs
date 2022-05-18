use std::convert::Infallible;
use std::env;
use std::process;

use hyper::service::{make_service_fn, service_fn};
use hyper::Server;

mod configuration;
mod service;

/// The name of the environment variable containing the path to the
/// configuration file.
const CONFIGURATION_FILE_ENV: &str = "RUST_ROLLER_CONFIGURATION_FILE";

#[tokio::main]
async fn main() {
    let path = match env::var(CONFIGURATION_FILE_ENV) {
        Ok(path) => path,
        Err(e) => {
            eprintln!(
                "environment variable {} not set: {}",
                CONFIGURATION_FILE_ENV, e,
            );
            process::exit(1);
        }
    };
    let configuration = match configuration::Configuration::load(&path) {
        Ok(configuration) => configuration,
        Err(e) => {
            eprintln!("failed to read configuation file {}: {}", path, e);
            process::exit(1);
        }
    };
    let context = match service::Context::try_from(&configuration) {
        Ok(configuration) => configuration,
        Err(e) => {
            eprintln!("failed to parse configuration: {}", e);
            process::exit(1);
        }
    };

    let service = make_service_fn(move |_conn| {
        let context = context.clone();
        let service =
            service_fn(move |req| service::handle(context.clone(), req));
        async move { Ok::<_, Infallible>(service) }
    });
    if let Err(e) = Server::bind(&configuration.bind).serve(service).await {
        eprintln!("server error: {}", e);
    }
}
