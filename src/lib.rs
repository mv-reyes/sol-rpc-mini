//! Minimal JSON-RPC client for Solana.
//!
//! The full `solana-sdk` dependency tree is heavy; most tools only need a
//! handful of read-only calls. This crate covers those with plain HTTP.

use serde_json::{json, Value};

pub mod det;
pub mod http;

pub const MAINNET: &str = "https://api.mainnet-beta.solana.com";

const B58: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

#[derive(Debug)]
pub struct RpcError(pub String);

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rpc error: {}", self.0)
    }
}

impl std::error::Error for RpcError {}

/// A read-only Solana JSON-RPC client.
pub struct RpcClient {
    endpoint: String,
}

impl RpcClient {
    pub fn new(endpoint: &str) -> Self {
        Self { endpoint: endpoint.to_string() }
    }

    pub fn mainnet() -> Self {
        Self::new(MAINNET)
    }

    /// Raw JSON-RPC call. Returns the `result` field.
    pub fn call(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        })
        .to_string();
        let (_, out) = http::request_json(
            "POST",
            &self.endpoint,
            &[("Content-Type", "application/json")],
            &body,
            20,
        )
        .map_err(|e| RpcError(e.to_string()))?;
        if let Some(err) = out.get("error") {
            return Err(RpcError(format!("{err}")));
        }
        Ok(out["result"].clone())
    }

    /// Account info with base64 data encoding.
    pub fn get_account(&self, pubkey: &str) -> Result<Value, RpcError> {
        self.call("getAccountInfo", json!([pubkey, { "encoding": "base64" }]))
    }

    /// Most recent signatures for an address (newest first).
    pub fn get_signatures(&self, pubkey: &str, limit: usize) -> Result<Value, RpcError> {
        self.call("getSignaturesForAddress", json!([pubkey, { "limit": limit }]))
    }

    /// Balance in lamports.
    pub fn get_balance(&self, pubkey: &str) -> Result<u64, RpcError> {
        let v = self.call("getBalance", json!([pubkey]))?;
        Ok(v["value"].as_u64().unwrap_or(0))
    }
}

/// Encode bytes as base58 (Bitcoin/Solana alphabet).
pub fn to_base58(data: &[u8]) -> String {
    let mut digits = vec![0u8];
    for &byte in data {
        let mut carry = byte as u32;
        for d in digits.iter_mut() {
            carry += (*d as u32) << 8;
            *d = (carry % 58) as u8;
            carry /= 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }
    let mut out = String::new();
    for _ in data.iter().take_while(|&&b| b == 0) {
        out.push('1');
    }
    for d in digits.iter().rev().skip_while(|&&d| d == 0) {
        out.push(B58[*d as usize] as char);
    }
    out
}

/// Decode a base58 string into bytes.
pub fn from_base58(s: &str) -> Result<Vec<u8>, RpcError> {
    let mut num: Vec<u8> = vec![0];
    for c in s.chars() {
        let val = B58
            .iter()
            .position(|&a| a as char == c)
            .ok_or_else(|| RpcError(format!("invalid base58 char {c}")))? as u32;
        let mut carry = val;
        for d in num.iter_mut() {
            let cur = (*d as u32) * 58 + carry;
            *d = (cur & 0xff) as u8;
            carry = cur >> 8;
        }
        while carry > 0 {
            num.push((carry & 0xff) as u8);
            carry >>= 8;
        }
    }
    let zeros = s.bytes().take_while(|&b| b == b'1').count();
    let mut out = vec![0u8; zeros];
    let be: Vec<u8> = num.into_iter().rev().collect();
    let start = be.iter().position(|&b| b != 0).unwrap_or(be.len());
    out.extend_from_slice(&be[start..]);
    Ok(out)
}
