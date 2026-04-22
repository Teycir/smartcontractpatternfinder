use serde::Serialize;
use std::net::SocketAddr;

pub const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:32145";
pub const SERVER_ADDR_ENV_VAR: &str = "SCPF_SERVER_ADDR";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    pub addr: String,
    pub origin: String,
}

pub fn server_config() -> ServerConfig {
    let addr =
        std::env::var(SERVER_ADDR_ENV_VAR).unwrap_or_else(|_| DEFAULT_SERVER_ADDR.to_string());
    server_config_from_addr(addr)
}

pub fn server_config_from_addr(addr: impl Into<String>) -> ServerConfig {
    let addr = addr.into();

    ServerConfig {
        origin: server_origin(&addr),
        addr,
    }
}

fn server_origin(addr: &str) -> String {
    let trimmed = addr
        .trim()
        .trim_start_matches("http://")
        .trim_end_matches('/');

    if let Ok(socket_addr) = trimmed.parse::<SocketAddr>() {
        format!("http://{}", socket_addr)
    } else {
        format!("http://{}", trimmed)
    }
}
