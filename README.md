# sorobanprj — Stellar dApps for Philippine SME money rails

Three demo-able [Soroban](https://developers.stellar.org/docs/build/smart-contracts/overview)
smart contracts built for real-world Philippine businesses: paying your team, paying your
freelancers, and getting paid across borders. Each contract is a standalone MVP with a working
transaction flow and tests. The full idea pitches live in [IDEAS.md](./IDEAS.md).

## Contracts

| Contract | Theme | MVP function | What it does |
|---|---|---|---|
| [`suweldo_chain`](./contracts/suweldo_chain) | SME payroll | `run_payroll(period)` | Batch-pays the whole roster from a USDC escrow in one transaction, writing an on-chain payslip per worker. |
| [`takdang_bayad`](./contracts/takdang_bayad) | Freelancer milestone escrow | `approve_milestone(job, m)` | Locks each milestone's USDC; releases to the freelancer on client approval, or refunds the client. |
| [`peso_bridge`](./contracts/peso_bridge) | Cross-border B2B | `pay_invoice(id)` | The buyer settles a USDC invoice straight to the exporter with a timestamped on-chain receipt. |

Each contract has its own `README.md` with the problem/solution, a sample CLI invocation, and
testnet deploy steps.

## Project Structure

```text
.
├── contracts
│   ├── suweldo_chain          # payroll
│   │   ├── src
│   │   │   ├── lib.rs
│   │   │   └── test.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── takdang_bayad          # freelancer milestone escrow
│   │   └── …
│   └── peso_bridge            # cross-border B2B invoicing
│       └── …
├── Cargo.toml                 # workspace: shared soroban-sdk dep + release profile
├── IDEAS.md                   # the three dApp idea pitches
├── LICENSE                    # MIT
└── README.md
```

- Each contract is a **workspace member** under `contracts/` with its own `Cargo.toml` that
  relies on the top-level `Cargo.toml` for the shared `soroban-sdk = "25"` dependency and the
  `[profile.release]` Wasm settings.
- Crates use `crate-type = ["cdylib", "rlib"]` — the `cdylib` builds the Wasm, the `rlib` lets
  `cargo test` build and run the tests natively.
- Add a new contract by creating another folder under `contracts/`; `members = ["contracts/*"]`
  in the root `Cargo.toml` picks it up automatically.

## Prerequisites

- **Rust** (stable) with the Wasm target: `rustup target add wasm32v1-none`
- **Stellar CLI** ≥ 22 (`stellar --version`) — `cargo install --locked stellar-cli`, or via Homebrew

## Build

```sh
# build every contract to Wasm
cargo build --target wasm32v1-none --release

# or build a single contract from its directory
cd contracts/suweldo_chain && stellar contract build
```

## Test

```sh
cargo test                       # all contracts (15 tests)
cargo test -p peso_bridge        # a single contract
```

## Deploy

Each contract's README has its own deploy + sample-invocation steps. The general flow:

```sh
stellar keys generate deployer --network testnet --fund

stellar contract deploy \
  --wasm target/wasm32v1-none/release/<contract_name>.wasm \
  --source deployer \
  --network testnet
```

## License

MIT — see [LICENSE](./LICENSE).
