# Archived Swap Implementations

This directory contains older/slower swap implementations kept for reference.

## Performance Comparison

| Implementation | Total Time | Notes |
|----------------|------------|-------|
| **helius-swap** (ACTIVE) | **1,525ms** | ⚡ Ultra API + Helius RPC - FASTEST |
| working-swap | 1,776ms | Ultra API + Default RPC |
| v6-swap | 2,092ms | V6 API + Helius RPC |
| tiny-swap | N/A | Early test implementation |
| debug-swap | N/A | Debugging/development version |
| raw-test | N/A | Raw API testing |

## Active Production Tool

Use this command for fastest swaps:
```bash
cargo run --bin swap
```

## Re-enabling Archived Versions

To use an archived version, uncomment the relevant binary in `Cargo.toml` and run:
```bash
cargo run --bin archive-{name}
```

## Benchmarking

All implementations tested with 0.001 SOL swaps to USDC on mainnet using Helius RPC where applicable.

**Winner:** Ultra API + Helius RPC = 1,525ms average
