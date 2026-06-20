# Takdang Bayad

> Milestone escrow for Philippine freelancers and their clients — funds release only when work is approved, settle instantly, no trust required.

## Problem & Solution

**Problem.** Marco, a freelance web developer in Davao, finishes a ₱60k build for a Manila
client who then ghosts on the final payment; with no escrow he eats the loss and weeks of
chasing — while clients equally fear paying upfront to a freelancer who might vanish.

**Solution.** Client and freelancer agree a milestone contract; the client locks each
milestone's USDC into this Soroban escrow, and funds release to the freelancer only on the
client's approval (or refund to the client if cancelled before approval). Stellar settles
the release instantly and the on-chain record is the dispute-proof receipt — neither side
trusts the other or a bank hold.

## Stellar Features Used

- **USDC transfers** — milestone funds escrowed and released in a SEP-41 token (USDC).
- **Soroban smart contract** — per-milestone fund/approve/refund state machine.
- **Trustlines** — client, contract, and freelancer hold the USDC asset.

## Core Feature (MVP)

`create_job(client, freelancer, [amounts])` →
`fund_milestone(job, m)` locks USDC in escrow →
`approve_milestone(job, m)` releases that USDC to the freelancer and marks it released.
Approving an unfunded or already-released milestone is rejected; `refund_milestone` returns
locked funds to the client if not yet approved. **Demoable in under 2 minutes.**

## Timeline (bootcamp)

- **Day 1:** contract (escrow state machine, fund/approve/refund) + tests — *this repo*.
- **Day 2:** deploy to testnet; minimal client/freelancer web view with a "Release" button.
- **Day 3:** add a dispute/arbiter address, polish, and a 2-minute live escrow demo.

## Vision & Purpose

Give the millions of Filipino freelancers a payment rail that protects both sides:
freelancers get guaranteed-funded milestones, clients pay only for approved work, and every
release is an instant, verifiable on-chain receipt — replacing trust and chargebacks with
code.

## Prerequisites

- **Rust** (stable) with the `wasm32v1-none` target: `rustup target add wasm32v1-none`
- **Stellar CLI** ≥ 22 (`stellar --version`) — installs via `cargo install --locked stellar-cli` or Homebrew
- `soroban-sdk` 25 (pinned in the workspace `Cargo.toml`)

## Build

```sh
stellar contract build
# or, from the workspace root:
cargo build --target wasm32v1-none --release -p takdang_bayad
```

## Test

```sh
cargo test -p takdang_bayad
```

## Deploy to testnet

```sh
stellar keys generate client --network testnet --fund

stellar contract deploy \
  --wasm target/wasm32v1-none/release/takdang_bayad.wasm \
  --source client \
  --network testnet
# → returns CONTRACT_ID
```

## Sample CLI invocation (MVP)

```sh
# Initialize with a USDC token contract id
stellar contract invoke --id CONTRACT_ID --source client --network testnet -- \
  initialize --token CUSDC...

# Open a job with two milestones (100 and 200), fund the first, then approve it
stellar contract invoke --id CONTRACT_ID --source client --network testnet -- \
  create_job --client GCLIENT... --freelancer GFREELANCER... --amounts '["100","200"]'

stellar contract invoke --id CONTRACT_ID --source client --network testnet -- \
  fund_milestone --job_id 1 --milestone 0

stellar contract invoke --id CONTRACT_ID --source client --network testnet -- \
  approve_milestone --job_id 1 --milestone 0
```

## License

MIT — see [LICENSE](../../LICENSE). Copyright (c) 2026 Takdang Bayad contributors.
