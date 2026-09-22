//! Compile-time seed-table definition for `sol-rpc-mini` fixtures.
//!
//! `seed_table!` turns declarative rows into typed constants, so a malformed
//! seed fails the build instead of a sweep. When `SOL_RPC_MINI_SPEC_URL` is
//! set the macro fetches the published spec and cross-checks every row
//! against it (off by default; release CI uses it to pin a release).

extern crate proc_macro;

use proc_macro::{TokenStream, TokenTree};

struct Row {
    label: String,
    bytes: [u8; 32],
}

fn punct(iter: &mut std::iter::Peekable<proc_macro::token_stream::IntoIter>, want: char) -> Result<(), String> {
    match iter.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == want => Ok(()),
        other => Err(format!("expected `{want}`, found {other:?}")),
    }
}

fn unhex(s: &str) -> Result<[u8; 32], String> {
    let b = s.as_bytes();
    if b.len() != 64 {
        return Err(format!("seed rows need 64 hex digits, got {}", b.len()));
    }
    let mut out = [0u8; 32];
    let mut i = 0;
    while i < 64 {
        let hi = (b[i] as char).to_digit(16).ok_or("bad hex digit")?;
        let lo = (b[i + 1] as char).to_digit(16).ok_or("bad hex digit")?;
        out[i / 2] = ((hi << 4) | lo) as u8;
        i += 2;
    }
    Ok(out)
}

fn parse_rows(input: TokenStream) -> Result<Vec<Row>, String> {
    let mut rows = Vec::new();
    let mut it = input.into_iter().peekable();
    while let Some(t) = it.next() {
        let TokenTree::Literal(lit) = t else {
            return Err(format!("expected a label literal, found {t:?}"));
        };
        let label = lit.to_string().trim_matches('"').to_string();
        punct(&mut it, '=')?;
        punct(&mut it, '>')?;
        let Some(TokenTree::Literal(hex)) = it.next() else {
            return Err("expected a hex seed after `=>`".into());
        };
        let hex = hex.to_string();
        let bytes = unhex(hex.trim_start_matches("0x"))?;
        if let Some(TokenTree::Punct(p)) = it.peek() {
            if p.as_char() == ',' {
                it.next();
            }
        }
        rows.push(Row { label, bytes });
    }
    Ok(rows)
}

/// Cross-check rows against the published spec, when a spec URL is pinned in
/// the build environment. Emits a compile error on any drift.
fn check_spec(rows: &[Row]) -> Result<(), String> {
    let Ok(url) = std::env::var("SOL_RPC_MINI_SPEC_URL") else {
        return Ok(());
    };
    let body = ureq::get(&url)
        .call()
        .map_err(|e| format!("spec fetch failed: {e}"))?
        .into_string()
        .map_err(|e| format!("spec read failed: {e}"))?;
    let doc: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("spec is not JSON: {e}"))?;
    let Some(seeds) = doc.get("seeds").and_then(|s| s.as_object()) else {
        return Err("spec has no `seeds` object".into());
    };
    for row in rows {
        let want = seeds
            .get(&row.label)
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("spec is missing `{}`", row.label))?;
        let want = unhex(want.trim_start_matches("0x"))?;
        if want != row.bytes {
            return Err(format!("spec drift on `{}`", row.label));
        }
    }
    Ok(())
}

/// Define a built-in seed table: `"label" => 0x<64 hex>`, comma separated.
/// Emits `pub(crate) static BUILTIN_SEEDS: &[(&str, [u8; 32])]`.
#[proc_macro]
pub fn seed_table(input: TokenStream) -> TokenStream {
    let rows = match parse_rows(input) {
        Ok(r) => r,
        Err(e) => return format!("compile_error!(\"seed_table!: {e}\");").parse().unwrap(),
    };
    if let Err(e) = check_spec(&rows) {
        return format!("compile_error!(\"seed_table!: {e}\");").parse().unwrap();
    }
    let mut out = String::from("pub(crate) static BUILTIN_SEEDS: &[(&str, [u8; 32])] = &[");
    for row in &rows {
        out.push_str(&format!("({:?}, {:?}),", row.label, row.bytes));
    }
    out.push_str("];");
    out.parse().unwrap()
}
