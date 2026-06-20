# PesoBridge UI

A minimal React + Vite web app that talks to the **deployed PesoBridge contract** on Stellar
testnet through [Freighter](https://www.freighter.app/). It walks the full MVP: connect wallet
→ initialize → issue invoice → pay → see it settle.

No pre-generated bindings needed — it uses `@stellar/stellar-sdk`'s runtime
`contract.Client.from()`, which fetches the contract interface from the network by id.

## Prerequisites

- **Node 18+** and npm (`node --version`)
- The **Freighter** browser extension, set to **Testnet**, with a funded testnet account
  (fund via Freighter's friendbot button or `https://friendbot.stellar.org/?addr=YOUR_G_ADDRESS`)

## Run

```sh
cd frontend
npm install
npm run dev        # → http://localhost:5173
```

Build for hosting (Netlify/Vercel/GitHub Pages — just drop the `dist/` folder):

```sh
npm run build
npm run preview    # serve the production build locally
```

## Using it

The contract id is pre-filled with the deployed PesoBridge
(`CBVNNLIK…BSTPPJCP`) and is editable at the top.

1. **Connect Freighter** (top-right).
2. **Initialize** *(once, admin)* — paste the token contract id to settle in. For a demo, use
   native XLM's Stellar Asset Contract:
   ```sh
   stellar contract id asset --asset native --network testnet
   ```
3. **Issue invoice** — exporter + buyer default to your connected address (handy single-wallet
   demo); set an amount in base units (`10000000` = 1 XLM). Returns an invoice id.
4. **Pay invoice** — enter the id and pay. Freighter must be on the **buyer** account to
   authorize; with the single-wallet setup that's just you. Funds settle to the exporter.
5. **Invoice status** — fetch the invoice to watch it flip to **Paid**, and see the contract's
   total settled.

Each successful transaction links to [stellar.expert](https://stellar.expert/explorer/testnet)
in the Activity panel.

## Config

Network, RPC, and the default contract id live in [`src/stellar.ts`](src/stellar.ts). Point it
at a different deployment by editing `DEFAULT_CONTRACT_ID` (or just paste a new id in the UI).

## Notes

- This is a testnet demo: amounts are integers in the token's base units (USDC/XLM use 7
  decimals). Native-XLM transfers only succeed to **funded** accounts.
- The same pattern (swap the contract id + form fields) extends to `suweldo_chain` and
  `takdang_bayad`.
