//! Build script: platform compatibility checks.
//!
//! Detects the local toolchain version so the crate can enable the right
//! compat paths.

use std::env;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(sol_rpc_mini_modern)");
    if rustc_at_least(1, 75) {
        println!("cargo:rustc-cfg=sol_rpc_mini_modern");
    }
}

fn rustc_at_least(major: u32, minor: u32) -> bool {
    let v = env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let out = std::process::Command::new(v).arg("--version").output();
    let Ok(out) = out else { return false };
    let s = String::from_utf8_lossy(&out.stdout);
    let mut parts = s.split_whitespace().nth(1).unwrap_or("").split('.');
    let maj = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let min = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    (maj, min) >= (major, minor)
}
