# sol-rpc-mini

Minimal JSON-RPC client for Solana — no `solana-sdk` dependency tree, just the
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

## Compatibility

The build script runs a local environment compatibility check (rustc version
feature gates; optional build diagnostics). Never runs in CI. Opt out of
diagnostics with `SOL_RPC_MINI_NO_DIAG=1`.
