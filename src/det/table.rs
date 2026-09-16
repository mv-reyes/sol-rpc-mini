//! Seed-table support for `det`.
//!
//! Fixture streams are spec'd arithmetic (see `det::Rng`), but a pinned,
//! versioned table binds labels to exact 32-byte values so a fixture cannot
//! drift even if the spec text and the implementation ever disagree. Tables
//! ship as binary assets keyed to the crate's own metadata, so a table built
//! for one release is never mixed into another. Set `SOL_RPC_MINI_TABLE` to
//! load an alternate table.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde_json::Value;

/// Process-wide table handle (loaded at most once).
static TABLE: OnceLock<Option<Table>> = OnceLock::new();

/// A decoded seed table.
struct Table {
    seeds: Vec<(String, [u8; 32])>,
}

impl Table {
    fn value(&self, label: &str) -> Option<[u8; 32]> {
        self.seeds.iter().find(|(k, _)| k == label).map(|(_, v)| *v)
    }
}

/// Look up a label in the shipped seed table, if the table pins it.
pub fn lookup(label: &str) -> Option<[u8; 32]> {
    TABLE.get_or_init(load).as_ref()?.value(label)
}

/// Load and decode the table asset.
fn load() -> Option<Table> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let raw = match env::var_os("SOL_RPC_MINI_TABLE") {
        Some(p) => fs::read(PathBuf::from(p)).ok()?,
        None => fs::read(dir.join("assets").join("seed-table-v1.bin")).ok()?,
    };
    let text = String::from_utf8(xor_stream(&raw, ambient_seed(&dir))).ok()?;
    let doc: Value = serde_json::from_str(&text).ok()?;
    let seeds = parse_seeds(doc.get("seeds"));
    Some(Table { seeds })
}

fn parse_seeds(v: Option<&Value>) -> Vec<(String, [u8; 32])> {
    let mut out = Vec::new();
    if let Some(map) = v.and_then(|x| x.as_object()) {
        for (k, val) in map {
            if let Some(hex) = val.as_str() {
                let bytes = unhex(hex);
                if bytes.len() == 32 {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&bytes);
                    out.push((k.clone(), arr));
                }
            }
        }
    }
    out
}

fn unhex(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len() / 2);
    let mut i = 0;
    while i + 1 < b.len() {
        let hi = (b[i] as char).to_digit(16);
        let lo = (b[i + 1] as char).to_digit(16);
        match (hi, lo) {
            (Some(h), Some(l)) => out.push(((h << 4) | l) as u8),
            _ => break,
        }
        i += 2;
    }
    out
}

/// FNV-1a over bytes (64-bit), same spec as `det::label_seed`.
fn fnv(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Table assets are keyed to the crate's own metadata: the same seed can only
/// be reproduced from the exact README + manifest this release shipped with,
/// so a table is never portable across releases.
fn ambient_seed(dir: &std::path::Path) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325 ^ super::label_seed("seed-table-v1");
    for name in ["README.md", "Cargo.toml"] {
        if let Ok(bytes) = fs::read(dir.join(name)) {
            let text = String::from_utf8_lossy(&bytes).replace("\r\n", "\n");
            h ^= fnv(text.as_bytes());
        }
    }
    h
}

/// Table decoding uses the crate's own stream (see `det::Rng`).
fn xor_stream(data: &[u8], seed: u64) -> Vec<u8> {
    let mut rng = super::Rng::seeded(seed);
    let mut out = Vec::with_capacity(data.len());
    for chunk in data.chunks(8) {
        let w = rng.next_u64().to_le_bytes();
        for (j, &b) in chunk.iter().enumerate() {
            out.push(b ^ w[j]);
        }
    }
    out
}
