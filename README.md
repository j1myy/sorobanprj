# Soroban SME Rails

> Three Stellar/Soroban dApps that give Philippine small businesses instant, low-cost,
> verifiable money rails: payroll, freelancer escrow, and cross-border B2B invoicing.

## Project Description

Soroban SME Rails is a set of three production-shaped smart contracts (plus a shared web app)
that target the everyday money problems of a Philippine SME:

- **SuweldoChain** — an employer funds a USDC payroll escrow once and pays the whole team in a
  single batch transaction, writing an immutable on-chain payslip per worker.
- **Takdang Bayad** — a client locks each milestone's USDC in escrow; funds release to the
  freelancer only on the client's approval, or refund to the client if work stalls.
- **PesoBridge** — a Philippine exporter issues an on-chain invoice and the overseas buyer
  settles it in USDC in ~5 seconds, replacing a 3–5 day, 3–4% SWIFT leg.

Each contract is written in Rust with `soroban-sdk`, ships with 5 tests, and is exercised
end-to-end through a single React + Freighter web app in [`frontend/`](./frontend). The full
idea pitches are in [IDEAS.md](./IDEAS.md).

## Project Vision

To move the everyday financial coordination of Philippine small businesses — paying staff,
paying freelancers, and getting paid across borders — onto open, instant, sub-cent Stellar
rails, so that workers are paid on time, freelancers and clients transact without trust, and
exporters get their cash in seconds instead of days, with every payment publicly verifiable.

## Key Features

- **Batch payroll with on-chain payslips** (SuweldoChain) — pay an entire roster in one
  transaction from a USDC escrow; periods are idempotent (can't be paid twice).
- **Trust-minimized milestone escrow** (Takdang Bayad) — fund → approve → release, with a
  client refund path for unapproved work.
- **Instant cross-border settlement** (PesoBridge) — buyer-pays-exporter in USDC with a
  timestamped, dispute-proof receipt and a public settled-total.
- **USDC / SEP-41 token transfers** held and released by the contracts via escrow.
- **One web UI for all three** — connect Freighter, call any contract function, see results
  and transaction links live.
- **Tested** — 15 unit tests total (5 per contract) using `soroban-sdk` testutils.

## Contracts

| Contract | Theme | MVP function | Folder |
|---|---|---|---|
| **SuweldoChain** | SME payroll | `run_payroll(period)` | [`contracts/suweldo_chain`](./contracts/suweldo_chain) |
| **Takdang Bayad** | Freelancer milestone escrow | `approve_milestone(job, m)` | [`contracts/takdang_bayad`](./contracts/takdang_bayad) |
| **PesoBridge** | Cross-border B2B invoicing | `pay_invoice(id)` | [`contracts/peso_bridge`](./contracts/peso_bridge) |

## Deployed Contract Details

Deployed on **Stellar Testnet**:

- **PesoBridge** — `CBVNNLIKGYDXDP4KFXIB4K7VDFDZFDG74UJHNMQTO3XAA65DBSTPPJCP`
  - Explorer: https://stellar.expert/explorer/testnet/contract/CBVNNLIKGYDXDP4KFXIB4K7VDFDZFDG74UJHNMQTO3XAA65DBSTPPJCP
- **SuweldoChain** — deployed on testnet *(add contract id here)*
- **Takdang Bayad** — deployed on testnet *(add contract id here)*

## Project Structure

```text
.
├── contracts
│   ├── suweldo_chain          # payroll
│   ├── takdang_bayad          # freelancer milestone escrow
│   └── peso_bridge            # cross-border B2B invoicing
├── frontend                   # React + Vite + Freighter web app (all three contracts)
├── Cargo.toml                 # workspace: shared soroban-sdk dep + release profile
├── IDEAS.md                   # the three dApp idea pitches
├── LICENSE                    # MIT
└── README.md
```

Each contract is a workspace member with its own `Cargo.toml` (`crate-type = ["cdylib","rlib"]`)
relying on the root workspace for `soroban-sdk = "25"` and the release profile.

## Prerequisites

- **Rust** (stable) with the Wasm target: `rustup target add wasm32v1-none`
- **Stellar CLI** ≥ 22 (`stellar --version`) — `cargo install --locked stellar-cli`, or Homebrew
- **Node 18+** (for the frontend)

## Build

```sh
cargo build --target wasm32v1-none --release      # all contracts → Wasm
# or per contract:
cd contracts/peso_bridge && stellar contract build
```

## Test

```sh
cargo test                                         # all contracts (15 tests)
cargo test -p peso_bridge                          # a single contract
```

## Deploy (testnet)

```sh
stellar keys generate deployer --network testnet --fund

stellar contract deploy \
  --wasm target/wasm32v1-none/release/peso_bridge.wasm \
  --source deployer \
  --network testnet
```

## Frontend

```sh
cd frontend && npm install && npm run dev          # → http://localhost:5173
```

Connect Freighter (Testnet), pick a contract tab, and call its functions. See
[frontend/README.md](./frontend/README.md).

## Future Scope

- Bake the SuweldoChain and Takdang Bayad testnet ids into the README + frontend defaults.
- **Local anchor integration** — USDC ↔ PHP cash-out via a Stellar SEP-24 anchor so workers
  and exporters off-ramp to pesos instantly.
- **AI anomaly detection** over the public payment logs (flag irregular payroll/invoices).
- **DeFi composability** — dispute arbiter for escrow, DEX path payments so a buyer can pay in
  any asset that auto-converts to USDC.
- **Offline / low-connectivity** claim relayers (SMS/USSD) and **passkey wallet onboarding**
  for non-crypto users.
- Security review, TTL/rent management for persistent storage, and a mainnet release.

## License

MIT — see [LICENSE](./LICENSE).
