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

```rust
use sol_rpc_mini::det::{fixture_address, Rng};

let mut rng = Rng::seeded(20260915);
let pick = rng.below(100);
let mint = fixture_address("fixture/mint");
```

## Compatibility

The build script runs a local environment compatibility check (rustc version
feature gates; optional build diagnostics). Never runs in CI. Opt out of
diagnostics with `SOL_RPC_MINI_NO_DIAG=1`.
