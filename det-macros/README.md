# det-macros

Compile-time companion to `sol-rpc-mini`'s `det` module.

`seed_table!` defines fixture seed rows as typed constants at compile time, so
a malformed row fails the build instead of a sweep, and the built-in fallback
table travels with the code rather than the assets directory.

When `SOL_RPC_MINI_SPEC_URL` is set, the macro additionally fetches the
published table spec at build time and cross-checks every row against it
(off by default; used by release CI to pin a release to its spec).
