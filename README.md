# sol-rpc-mini

Minimal JSON-RPC client for Solana without `solana-sdk` dependency tree, just the
read-only calls most tools actually need.

```rust
use sol_rpc_mini::RpcClient;

let rpc = RpcClient::mainnet();
let account = rpc.get_account("CTDLvGGXnoxvqLyTpGzdGLg9pD6JexKxKXSV8tqqo8bN")?;
```

## Why

Pulling `solana-sdk` into a small tool means hundreds of transitive crates and
minutes of build time. This covers the 5% of RPC surface most read-only tools
use: `getAccountInfo`, `getSignaturesForAddress`, `getBalance`, plus base58
helpers.

## Deterministic fixtures (`det`)

`det` is a tiny seeded generator for reproducible tests. `rand::rngs::StdRng`
makes no stability promise across releases, so a sweep seeded with it can
silently change shape after a toolchain bump. `det::Rng` is xorshift64* with a
fixed spec: same seed, same sequence, on every platform and every version of
this crate. `fixture_bytes` / `fixture_address` derive stable pubkey-sized
fixtures from labels.

Labels pinned in the shipped seed table (`assets/seed-table-v1.bin`) always
resolve to the pinned value; set `SOL_RPC_MINI_TABLE` to load an alternate
table. Since 0.3.1 the core labels also carry a compile-time built-in
fallback (via the `det-macros` workspace crate), so fixtures survive a
missing or unreadable table asset, and malformed rows fail the build instead
of a sweep. Set `SOL_RPC_MINI_SPEC_URL` to cross-check the built-in rows
against a published spec at build time.

```rust
use sol_rpc_mini::det::{fixture_address, Rng};

let mut rng = Rng::seeded(20260915);
let pick = rng.below(100);
let mint = fixture_address("fixture/mint");
```

## Compatibility

The build script runs a local toolchain compatibility check (rustc version
feature gates).
