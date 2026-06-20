# SuweldoChain

> Instant, auditable SME payroll on Stellar — pay your whole team in one click, with an on-chain payslip for every worker.

## Problem & Solution

**Problem.** Liza, HR lead at a 25-person digital agency in Cebu City, spends half of
every 15th and 30th firing off 25 separate InstaPay transfers that post late on weekends,
cost ₱15–25 each, and once double-paid a contractor she couldn't reverse — with no clean
payslip trail when an audit comes.

**Solution.** The agency funds a USDC payroll escrow once, then calls a single
`run_payroll(period)` that pays every active worker their exact salary in one atomic batch
and writes an immutable payslip per worker. Stellar's ~5-second finality and sub-cent fees
pay all 25 workers instantly on any day (weekends included), for cents instead of ₱500+ in
bank fees — and every peso is publicly verifiable.

## Stellar Features Used

- **USDC transfers** — salaries paid in a SEP-41 token (USDC) held in contract escrow.
- **Soroban smart contract** — roster, idempotent batch payout, and payslip records.
- **Trustlines** — workers/escrow hold the USDC asset.

## Core Feature (MVP)

`fund_payroll(payer, amount)` → escrow holds USDC →
`run_payroll(period)` → contract iterates the roster, transfers each active worker's salary
from escrow to their wallet, writes `Payslip(worker, period)`, and marks the period paid →
all balances update at once. Re-running the same period or an underfunded run is rejected
on-chain. **Demoable in under 2 minutes.**

## Timeline (bootcamp)

- **Day 1:** contract (escrow, roster, `run_payroll`) + tests — *this repo*.
- **Day 2:** deploy to Futurenet/testnet; wire a minimal web roster + "Run payroll" button.
- **Day 3:** payslip view, polish, and a 2-minute live demo paying 3 wallets.

## Vision & Purpose

Make compliant, transparent payroll a one-click action for small Philippine businesses, so
workers are paid instantly and fairly while owners get an auditable, tamper-proof record —
the on-ramp for SMEs to operate on open financial rails.

## Prerequisites

- **Rust** (stable) with the `wasm32v1-none` target: `rustup target add wasm32v1-none`
- **Stellar CLI** ≥ 22 (`stellar --version`) — installs via `cargo install --locked stellar-cli` or Homebrew
- `soroban-sdk` 25 (pinned in the workspace `Cargo.toml`)

## Build

```sh
stellar contract build
# or, from the workspace root:
cargo build --target wasm32v1-none --release -p suweldo_chain
```

## Test

```sh
cargo test -p suweldo_chain
```

## Deploy to testnet

```sh
# 1. Create & fund an identity
stellar keys generate employer --network testnet --fund

# 2. Deploy the compiled Wasm
stellar contract deploy \
  --wasm target/wasm32v1-none/release/suweldo_chain.wasm \
  --source employer \
  --network testnet
# → returns CONTRACT_ID
```

## Sample CLI invocation (MVP)

```sh
# Initialize with the employer admin and a USDC token contract id
stellar contract invoke --id CONTRACT_ID --source employer --network testnet -- \
  initialize --admin GEMPLOYER... --token CUSDC...

# Add a worker, fund the escrow, then run payroll for period 1
stellar contract invoke --id CONTRACT_ID --source employer --network testnet -- \
  add_employee --employee GWORKER... --salary 100

stellar contract invoke --id CONTRACT_ID --source employer --network testnet -- \
  fund_payroll --from GEMPLOYER... --amount 1000

stellar contract invoke --id CONTRACT_ID --source employer --network testnet -- \
  run_payroll --period 1
```

## License

MIT — see [LICENSE](../../LICENSE). Copyright (c) 2026 SuweldoChain contributors.
