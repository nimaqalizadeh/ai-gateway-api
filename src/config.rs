use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Deserialize, Clone, Debug)]
pub struct Settings {
    #[serde(default = "default_bind")]
    pub bind: SocketAddr,
}

fn default_bind() -> SocketAddr {
    "0.0.0.0:8080"
        .parse()
        .expect("hardcoded bind addr is valid")
}
