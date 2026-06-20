# Soroban SME Rails — web UI

One React + Vite app for **all three** deployed contracts on Stellar testnet, with a tab to
switch between them. It talks to each contract through [Freighter](https://www.freighter.app/)
using `@stellar/stellar-sdk`'s runtime `contract.Client.from()` — no pre-generated bindings,
just the contract id.

- 🌉 **PesoBridge** — cross-border B2B invoicing (issue → pay → settle)
- ⚡ **SuweldoChain** — SME payroll (add employees → fund → run payroll)
- 🤝 **Takdang Bayad** — freelancer milestone escrow (create job → fund → approve)

Every contract function is rendered as a small form (read calls simulate; write calls sign
with Freighter and submit), so the whole surface of each contract is clickable.

## Prerequisites

- **Node 18+** and npm
- The **Freighter** extension, set to **Testnet**, with a funded testnet account
  (fund via Freighter's friendbot button or `https://friendbot.stellar.org/?addr=YOUR_G_ADDRESS`)

## Run

```sh
cd frontend
npm install
npm run dev        # → http://localhost:5173
```

Build for hosting (Netlify/Vercel/GitHub Pages — drop the `dist/` folder):

```sh
npm run build
npm run preview
```

## Contract ids

- **PesoBridge** is pre-filled (`CBVNNLIK…BSTPPJCP`).
- **SuweldoChain** and **Takdang Bayad** start blank — paste each deployed id into the
  "Contract ID" box on its tab. Ids are saved to `localStorage`, so you only paste once. (To
  bake them in as defaults, set `defaultId` in [`src/contracts.ts`](src/contracts.ts).)

## Using it (example: PesoBridge)

1. **Connect Freighter** (top-right).
2. Pick the **PesoBridge** tab; its id is already set.
3. `initialize` → paste a token id. For a demo use native XLM's SAC:
   `stellar contract id asset --asset native --network testnet`.
4. `issue_invoice` → use the **me** chip to fill exporter/buyer with your address; amount in
   base units (`10000000` = 1 XLM). Returns an invoice id.
5. `pay_invoice` → enter the id (Freighter signs as the buyer).
6. `get_invoice` → watch the status flip to **Paid**; `total_settled` shows the amount.

SuweldoChain and Takdang Bayad follow the flow shown at the top of their tab.

## Project map

```
frontend/
├── index.html
├── src/
│   ├── contracts.ts   # declarative config: every contract + function (drives the forms)
│   ├── stellar.ts     # SDK client + Freighter + arg conversion helpers
│   ├── InvokeCard.tsx # one function → one form + result renderer
│   ├── App.tsx        # tabs, contract-id management, activity log
│   └── styles.css
└── package.json
```

Adding another contract is just one more entry in `CONTRACTS` in `src/contracts.ts`.

## Notes

- Testnet demo: amounts are integers in the token's base units (USDC/XLM use 7 decimals).
  Native-XLM transfers only succeed to **funded** accounts.
- Network, RPC, and PesoBridge's default id live in [`src/stellar.ts`](src/stellar.ts).
