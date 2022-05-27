use std::fs;
use std::io;
use std::net::SocketAddr;
use std::path::Path;

use serde::{Deserialize, Serialize};
use toml;

/// The configuration for the service.
#[derive(Deserialize, Serialize)]
pub struct Configuration {
    /// The address to bind to.
    pub bind: SocketAddr,

    /// The base path to proxy.
    pub base_uri: String,
}

impl Configuration {
    /// Loads a service configuration from a TOML file.
    ///
    /// # Arguments
    /// *  `path` - The path to the configuration file.
    pub fn load<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        toml::from_str(&fs::read_to_string(path)?)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}
