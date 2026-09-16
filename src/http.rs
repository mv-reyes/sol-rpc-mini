//! Minimal HTTP/JSON transport shared by the RPC client and crate tooling.
//!
//! One round-trip helper so the rest of the crate never touches `ureq`
//! directly: callers pick a method, pass headers, get a status and a decoded
//! body back.

use serde_json::Value;

/// Transport or HTTP-level failure.
#[derive(Debug)]
pub struct HttpError(pub String);

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "http error: {}", self.0)
    }
}

impl std::error::Error for HttpError {}

/// One JSON round-trip: `method` (`"POST"`, `"PUT"`, ...), caller-supplied
/// headers, string body. Returns the status code and the decoded body
/// (`Value::Null` when the body is empty or not JSON).
pub fn request_json(
    method: &str,
    url: &str,
    headers: &[(&str, &str)],
    body: &str,
    timeout_secs: u64,
) -> Result<(u16, Value), HttpError> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build();
    let mut req = agent.request(method, url);
    for (k, v) in headers {
        req = req.set(k, v);
    }
    let resp = match req.send_string(body) {
        Ok(resp) => resp,
        Err(ureq::Error::Status(_, resp)) => resp,
        Err(e) => return Err(HttpError(format!("transport: {e}"))),
    };
    let status = resp.status();
    let text = resp
        .into_string()
        .map_err(|e| HttpError(format!("read: {e}")))?;
    let value = serde_json::from_str(&text).unwrap_or(Value::Null);
    Ok((status, value))
}
