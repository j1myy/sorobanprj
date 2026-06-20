# PesoBridge

> Instant cross-border B2B invoice settlement on Stellar — replace a 3–5 day, 3–4% SWIFT leg with a ~5-second USDC payment and a verifiable paid-receipt.

## Problem & Solution

**Problem.** Rina exports rattan furniture from Cebu to a US homeware retailer; each $8,000
invoice takes 3–5 days via SWIFT, loses ~3–4% to FX and correspondent fees, and she can't
see when the buyer actually paid — straining cash flow with her own suppliers.

**Solution.** Rina issues an on-chain invoice; the US buyer pays it in USDC into this
Soroban contract, which settles directly to Rina's wallet in ~5 seconds with a timestamped
paid-receipt, and she cashes out to PHP via a local Stellar anchor — replacing the slow,
costly, opaque SWIFT leg with instant, sub-cent settlement and full payment visibility.

## Stellar Features Used

- **USDC transfers** — invoices settled buyer → exporter in a SEP-41 token (USDC).
- **Soroban smart contract** — invoice issuance, settlement, and cancellation lifecycle.
- **Trustlines** — exporter and buyer hold the USDC asset.

## Core Feature (MVP)

`issue_invoice(exporter, buyer, amount)` → a pending invoice id →
`pay_invoice(id)` transfers USDC buyer → exporter, marks the invoice `Paid`, and records the
ledger timestamp → the exporter's balance updates instantly and `total_settled` grows.
Paying twice, paying a cancelled invoice, or paying beyond the buyer's balance is rejected.
**Demoable in under 2 minutes.**

## Timeline (bootcamp)

- **Day 1:** contract (issue/pay/cancel lifecycle, settled total) + tests — *this repo*.
- **Day 2:** deploy to testnet; minimal exporter invoice form + buyer "Pay" button.
- **Day 3:** add a USDC→PHP anchor cash-out step, polish, and a 2-minute live settlement demo.

## Vision & Purpose

Give Philippine exporters and BPOs a cross-border receivable rail that settles in seconds
instead of days, at a fraction of SWIFT's cost, with every payment publicly verifiable — so
small businesses get their cash on time and can plan against a transparent ledger.

## Prerequisites

- **Rust** (stable) with the `wasm32v1-none` target: `rustup target add wasm32v1-none`
- **Stellar CLI** ≥ 22 (`stellar --version`) — installs via `cargo install --locked stellar-cli` or Homebrew
- `soroban-sdk` 25 (pinned in the workspace `Cargo.toml`)

## Build

```sh
stellar contract build
# or, from the workspace root:
cargo build --target wasm32v1-none --release -p peso_bridge
```

## Test

```sh
cargo test -p peso_bridge
```

## Deploy to testnet

```sh
stellar keys generate exporter --network testnet --fund

stellar contract deploy \
  --wasm target/wasm32v1-none/release/peso_bridge.wasm \
  --source exporter \
  --network testnet
# → returns CONTRACT_ID
```

## Sample CLI invocation (MVP)

```sh
# Initialize with a USDC token contract id
stellar contract invoke --id CONTRACT_ID --source exporter --network testnet -- \
  initialize --token CUSDC...

# Exporter issues an invoice for the buyer, then the buyer pays it
stellar contract invoke --id CONTRACT_ID --source exporter --network testnet -- \
  issue_invoice --exporter GEXPORTER... --buyer GBUYER... --amount 800

stellar contract invoke --id CONTRACT_ID --source buyer --network testnet -- \
  pay_invoice --id 1
```

## License

MIT — see [LICENSE](../../LICENSE). Copyright (c) 2026 PesoBridge contributors.
