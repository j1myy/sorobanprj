# 3 Stellar dApp Ideas — Philippine SME Money Rails

Three demo-able Stellar dApps built around the **core money rails of a Philippine business**:
paying your team, paying your freelancers, and getting paid across borders. Each maps 1:1 to a
Soroban contract in [`contracts/`](./contracts) with a working MVP and 5 tests.

| # | Project | Theme | Contract | MVP function |
|---|---------|-------|----------|--------------|
| 1 | **SuweldoChain** | Payroll & salaries | [`suweldo_chain`](./contracts/suweldo_chain) | `run_payroll(period)` |
| 2 | **Takdang Bayad** | Escrow for contracts / freelancer invoicing | [`takdang_bayad`](./contracts/takdang_bayad) | `approve_milestone(job, m)` |
| 3 | **PesoBridge** | Cross-border B2B payments | [`peso_bridge`](./contracts/peso_bridge) | `pay_invoice(id)` |

---

## 1. SuweldoChain ⚡

**Project name:** SuweldoChain

**Problem.** Liza, HR lead at a 25-person digital agency in Cebu City, spends half of every
15th and 30th firing off 25 separate InstaPay transfers that post late on weekends, cost
₱15–25 each, and once double-paid a contractor she couldn't reverse — with no clean payslip
trail when an audit comes.

**Solution.** The agency funds a USDC payroll escrow once and runs a single
`run_payroll(period)` that pays every active worker their exact salary in one atomic batch
on-chain, writing an immutable payslip per worker; Stellar's ~5-second finality and sub-cent
fees pay all 25 workers instantly on any day, for cents instead of ₱500+ in bank fees.

**Stellar features:** USDC transfers · Soroban smart contracts · Trustlines.

**Target users:** SME owners / HR officers at 5–50-staff agencies, BPOs, and shops in Cebu
(and nationwide) who pay salaried + contract workers and feel the per-transfer fees, weekend
delays, and missing audit trail.

**Core feature (MVP).** *User action:* HR funds the escrow and clicks "Run payroll." →
*On-chain:* the contract iterates the roster, transfers each active salary from escrow to the
worker, writes `Payslip(worker, period)`, and marks the period paid. → *Result:* every
worker's wallet is funded at once and the period can't be paid twice. (< 2 min demo.)

**Why this wins.** Real SME pain, real payroll money movement, and an instant + auditable
batch rail — exactly the "real users / local economy" story Stellar judges look for, with
on-chain payslips as the composable hook.

**Optional edge.** Local **anchor integration** — workers cash USDC → PHP instantly via a
Stellar anchor (e.g. a Coins.ph-style SEP-24 anchor).

---

## 2. Takdang Bayad 🤝

**Project name:** Takdang Bayad

**Problem.** Marco, a freelance web developer in Davao, finishes a ₱60k build for a Manila
client who then ghosts on the final payment; with no escrow he eats the loss and weeks of
chasing — while clients equally fear paying upfront to a freelancer who might vanish.

**Solution.** Client and freelancer agree a milestone contract; the client locks each
milestone's USDC into a Soroban escrow and funds release to the freelancer only on the
client's approval (or refund to the client if cancelled before approval); Stellar settles the
release instantly and the on-chain record is the dispute-proof receipt — neither side trusts
the other or a bank hold.

**Stellar features:** USDC transfers · Soroban smart contracts · Trustlines.

**Target users:** Philippine freelancers (devs, designers, virtual assistants) and their
local/foreign clients, who both carry non-payment / non-delivery risk on every gig.

**Core feature (MVP).** *User action:* client funds milestone 0, then approves it when the
work lands. → *On-chain:* `fund_milestone` locks USDC in escrow; `approve_milestone`
transfers it to the freelancer and marks it released. → *Result:* the freelancer is paid
instantly; an unfunded or already-released milestone is rejected, and `refund_milestone`
returns funds if work stalls. (< 2 min demo.)

**Why this wins.** Two real users, a trust-minimized escrow primitive, and instant settlement
— a textbook Stellar escrow composability story with a clear local-economy impact.

**Optional edge.** **DeFi composability** — an arbiter address for disputes and/or released
USDC swappable via Stellar's built-in DEX.

---

## 3. PesoBridge 🌉

**Project name:** PesoBridge

**Problem.** Rina exports rattan furniture from Cebu to a US homeware retailer; each $8,000
invoice takes 3–5 days via SWIFT, loses ~3–4% to FX and correspondent fees, and she can't see
when the buyer actually paid — straining cash flow with her own suppliers.

**Solution.** Rina issues an on-chain invoice; the US buyer pays it in USDC into a Soroban
contract that settles directly to her wallet in ~5 seconds with a timestamped paid-receipt,
and she cashes out to PHP via a local Stellar anchor — replacing the slow, costly, opaque
SWIFT leg with instant, sub-cent settlement and full payment visibility.

**Stellar features:** USDC transfers · Soroban smart contracts · Trustlines.

**Target users:** Philippine SME exporters and BPOs invoicing foreign clients, plus their
overseas buyers, all stuck with slow and expensive correspondent banking.

**Core feature (MVP).** *User action:* exporter issues an invoice; the buyer clicks "Pay." →
*On-chain:* `pay_invoice` transfers USDC buyer → exporter, marks the invoice `Paid`, and
stamps the ledger timestamp. → *Result:* the exporter is paid instantly, `total_settled`
grows, and paying twice / a cancelled invoice / beyond balance is rejected. (< 2 min demo.)

**Why this wins.** Real exporter cash-flow pain, real cross-border money movement, and
instant + transparent settlement — Stellar's flagship remittance/B2B strength shown live.

**Optional edge.** Local **anchor integration** for USDC↔PHP cash-out (and/or a DEX path
payment so the buyer can pay in another asset that auto-converts to USDC).

---

## Build & test everything

```sh
# from the workspace root
cargo test                                   # all three contracts' tests (15 total)
cargo build --target wasm32v1-none --release # build all to Wasm
```

Per-contract build/test/deploy steps and a sample CLI invocation live in each contract's
`README.md`. All three are workspace members under `contracts/`, sharing `soroban-sdk = "25"`
and the release profile from the root `Cargo.toml`.

> **Out of scope (by design):** front-end dashboards, AI anomaly detection, and anchor
> cash-out are listed as each idea's *optional edge* and are natural day-2/3 additions — the
> contracts here are the on-chain core that proves each MVP end-to-end.
