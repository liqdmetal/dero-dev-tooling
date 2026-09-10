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
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

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
    /// The endpoint or HTTP response was malformed.
    #[error("invalid rpc endpoint/response: {0}")]
    InvalidResponse(String),
}

/// A validated DERO daemon RPC client.
#[derive(Debug, Clone)]
pub struct Client {
    url: String,
    host: String,
    port: u16,
    path: String,
    timeout: Duration,
    max_response_bytes: usize,
}

impl Client {
    /// Connect to a DERO daemon JSON-RPC endpoint.
    pub fn new(host: &str, port: u16) -> Result<Self, RpcError> {
        if host.is_empty() || host.chars().any(|c| c.is_ascii_whitespace() || c == '/') {
            return Err(RpcError::InvalidResponse("invalid host".into()));
        }
        let display_host = if host.contains(':') && !host.starts_with('[') {
            format!("[{host}]")
        } else {
            host.to_string()
        };
        Ok(Self {
            url: format!("http://{display_host}:{port}/json_rpc"),
            host: host.trim_matches(['[', ']']).to_string(),
            port,
            path: "/json_rpc".to_string(),
            timeout: DEFAULT_TIMEOUT,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
        })
    }

    /// Return the validated endpoint URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Set the connect/read/write timeout for subsequent calls.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the maximum HTTP response body size.
    pub fn with_max_response_bytes(mut self, max_bytes: usize) -> Self {
        self.max_response_bytes = max_bytes;
        self
    }

    /// Query the balance of a DERO address on the daemon chain.
    ///
    /// Uses `get_balance` with the standard scid argument. Returns the raw
    /// result JSON so the caller can decode it; the daemon must be running.
    /// This is a read-only prepare step.
    pub fn get_balance(&self, address: &str) -> Result<Value, RpcError> {
        let params = Arguments(vec![
            Argument { name: "address".into(), data_type: 1, value: ArgValue::Str(address.into()) },
            Argument { name: "scid".into(), data_type: 1, value: ArgValue::Str("0".repeat(64)) },
        ]);
        self.call("get_balance", &params)
    }

    /// Call a daemon method with `Arguments` params.
    pub fn call(&self, method: &str, params: &Arguments) -> Result<Value, RpcError> {
        if method.is_empty() || method.chars().any(|c| c.is_ascii_whitespace()) {
            return Err(RpcError::InvalidResponse("invalid method name".into()));
        }
        let param_bytes = params.marshal_binary().map_err(RpcError::Encode)?;
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": { "args": hex::encode(&param_bytes) },
        });
        let body =
            serde_json::to_string(&body).map_err(|error| RpcError::Transport(error.to_string()))?;
        let resp = http_post(
            &self.host,
            self.port,
            &self.path,
            &body,
            self.timeout,
            self.max_response_bytes,
        )
        .map_err(RpcError::Transport)?;
        let v: Value = serde_json::from_str(&resp)
            .map_err(|error| RpcError::InvalidResponse(error.to_string()))?;
        if let Some(err) = v.get("error") {
            return Err(RpcError::Daemon(err.to_string()));
        }
        Ok(v)
    }
}

/// Minimal bounded HTTP POST using std TCP. DERO's daemon endpoint is local
/// HTTP; HTTPS is intentionally not inferred from an arbitrary input URL.
fn http_post(
    host: &str,
    port: u16,
    path: &str,
    body: &str,
    timeout: Duration,
    max_response_bytes: usize,
) -> Result<String, String> {
    if max_response_bytes == 0 {
        return Err("response limit must be non-zero".into());
    }
    let mut addrs = (host, port)
        .to_socket_addrs()
        .map_err(|error| format!("resolve {host}:{port}: {error}"))?;
    let addr = addrs.next().ok_or_else(|| "endpoint has no addresses".to_string())?;
    let mut stream = TcpStream::connect_timeout(&addr, timeout)
        .map_err(|error| format!("connect {host}:{port}: {error}"))?;
    stream.set_read_timeout(Some(timeout)).map_err(|e| e.to_string())?;
    stream.set_write_timeout(Some(timeout)).map_err(|e| e.to_string())?;

    let host_header = if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]")
    } else {
        host.to_string()
    };
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {host_header}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;

    let mut raw = Vec::with_capacity(4096);
    let mut chunk = [0u8; 8192];
    loop {
        let read = stream.read(&mut chunk).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        if raw.len().saturating_add(read) > max_response_bytes {
            return Err("HTTP response exceeds configured limit".into());
        }
        raw.extend_from_slice(&chunk[..read]);
    }
    let (headers, body) =
        raw.split_once_bytes(b"\r\n\r\n").ok_or_else(|| "malformed HTTP response".to_string())?;
    let headers_text = std::str::from_utf8(headers).map_err(|_| "invalid HTTP headers")?;
    let status = headers_text
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| "missing HTTP status".to_string())?;
    if !(200..300).contains(&status) {
        return Err(format!("daemon returned HTTP status {status}"));
    }
    let is_chunked = headers_text.lines().any(|line| {
        line.split_once(':')
            .map(|(name, value)| {
                name.eq_ignore_ascii_case("transfer-encoding")
                    && value.trim().eq_ignore_ascii_case("chunked")
            })
            .unwrap_or(false)
    });
    let decoded = if is_chunked { decode_chunked(body)? } else { body.to_vec() };
    String::from_utf8(decoded).map_err(|_| "invalid UTF-8 response body".into())
}

trait SplitOnceBytes {
    fn split_once_bytes(&self, needle: &[u8]) -> Option<(&[u8], &[u8])>;
}

impl SplitOnceBytes for [u8] {
    fn split_once_bytes(&self, needle: &[u8]) -> Option<(&[u8], &[u8])> {
        self.windows(needle.len())
            .position(|window| window == needle)
            .map(|i| (&self[..i], &self[i + needle.len()..]))
    }
}

fn decode_chunked(mut input: &[u8]) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    loop {
        let line_end = input
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or_else(|| "malformed chunk size".to_string())?;
        let size_text =
            std::str::from_utf8(&input[..line_end]).map_err(|_| "invalid chunk size")?;
        let size_text = size_text.split(';').next().unwrap_or_default().trim();
        let size = usize::from_str_radix(size_text, 16).map_err(|_| "invalid chunk size")?;
        input = &input[line_end + 2..];
        if size == 0 {
            return Ok(output);
        }
        if input.len() < size + 2 || &input[size..size + 2] != b"\r\n" {
            return Err("malformed chunk body".into());
        }
        output.extend_from_slice(&input[..size]);
        input = &input[size + 2..];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_builds_url() {
        let c = Client::new("127.0.0.1", 20209).expect("valid endpoint");
        assert_eq!(c.url(), "http://127.0.0.1:20209/json_rpc");
    }

    #[test]
    fn rejects_invalid_host() {
        assert!(Client::new("bad host", 20209).is_err());
        assert!(Client::new("", 20209).is_err());
    }

    #[test]
    fn arguments_encode() {
        let args = Arguments(vec![Argument {
            name: "scid".into(),
            data_type: 1,
            value: ArgValue::Str("0000".into()),
        }]);
        let bytes = args.marshal_binary().expect("marshal ok");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn decodes_chunked_body() {
        let body = b"4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n";
        assert_eq!(decode_chunked(body).unwrap(), b"Wikipedia");
    }
}
