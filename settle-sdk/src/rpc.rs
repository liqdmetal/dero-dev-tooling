//! Typed RPC client for a DERO daemon (mainnet), for preparing confidential
//! settlements against the live chain. Uses dero-crypto's `Arguments`
//! marshaling so parameters are encoded exactly as the daemon expects.
//!
//! NOTE: this is a read-only/prepare client — it queries and builds requests.
//! Broadcasting a transaction requires a signed tx and is intentionally left
//! to the caller (per project convention, chain-touching is gated).

use dero_rs::rpc_args::{ArgValue, Argument, Arguments};
use serde_json::Value;
use std::io::{Read, Write};

/// Error from the RPC client.
#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    /// Connection/transport failure.
    #[error("rpc transport: {0}")]
    Transport(String),
    /// The daemon returned an error.
    #[error("rpc error: {0}")]
    Daemon(String),
    /// Failed to encode parameters.
    #[error("encode: {0}")]
    Encode(String),
}

/// A DERO daemon RPC client.
#[derive(Debug, Clone)]
pub struct Client {
    /// Base URL, e.g. http://127.0.0.1:20209/json_rpc
    pub url: String,
}

impl Client {
    /// Connect to a DERO daemon JSON-RPC endpoint.
    pub fn new(host: &str, port: u16) -> Self {
        Self { url: format!("http://{host}:{port}/json_rpc") }
    }

    /// Call a daemon method with `Arguments` params.
    pub fn call(&self, method: &str, params: &Arguments) -> Result<Value, RpcError> {
        // Marshal params to the daemon's binary form for the body.
        let param_bytes = params
            .marshal_binary()
            .map_err(|e| RpcError::Encode(e))?;
        // Simple JSON-RPC wrapper (hex body).
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": { "args": hex::encode(&param_bytes) },
        });

        let resp = http_post(&self.url, &serde_json::to_string(&body).unwrap_or_default())
            .map_err(RpcError::Transport)?;
        let v: Value = serde_json::from_str(&resp).map_err(|e| RpcError::Transport(e.to_string()))?;
        if let Some(err) = v.get("error") {
            return Err(RpcError::Daemon(err.to_string()));
        }
        Ok(v)
    }
}

/// Minimal blocking HTTP POST (no external HTTP dep; uses std TCP).
fn http_post(url: &str, body: &str) -> Result<String, String> {
    // Parse http://host:port/path
    let rest = url.strip_prefix("http://").ok_or("only http:// supported")?;
    let (hostport, path) = match rest.split_once('/') {
        Some((hp, p)) => (hp, format!("/{p}")),
        None => (rest, "/".to_string()),
    };
    let (host, port) = hostport
        .split_once(':')
        .map(|(h, p)| (h.to_string(), p.parse::<u16>().unwrap_or(80)))
        .unwrap_or_else(|| (hostport.to_string(), 80));

    let mut stream = std::net::TcpStream::connect((host.as_str(), port))
        .map_err(|e| format!("connect {host}:{port}: {e}"))?;
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    let mut resp = String::new();
    stream.read_to_string(&mut resp).map_err(|e| e.to_string())?;
    // Split headers from body
    resp.split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .ok_or_else(|| "malformed response".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_builds_url() {
        let c = Client::new("127.0.0.1", 20209);
        assert_eq!(c.url, "http://127.0.0.1:20209/json_rpc");
    }

    #[test]
    fn arguments_encode() {
        let args = Arguments(vec![Argument {
            name: "scid".into(),
            data_type: 1, // string type tag (matches daemon expectation)
            value: ArgValue::Str("0000".into()),
        }]);
        let bytes = args.marshal_binary().expect("marshal ok");
        assert!(!bytes.is_empty());
    }
}
