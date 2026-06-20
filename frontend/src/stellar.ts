// Thin wrapper around @stellar/stellar-sdk's contract client + Freighter wallet.
//
// We use the runtime `contract.Client.from(...)`, which fetches each contract's
// interface from the network — so this app needs no pre-generated bindings, just
// the deployed contract id.

import {
  requestAccess,
  getAddress,
  signTransaction as freighterSign,
} from '@stellar/freighter-api';
import * as StellarSdk from '@stellar/stellar-sdk';
import type { ArgType } from './contracts';

export const NETWORK_PASSPHRASE = 'Test SDF Network ; September 2015';
export const RPC_URL = 'https://soroban-testnet.stellar.org';

// Your deployed PesoBridge contract on testnet.
export const DEFAULT_PESO_BRIDGE_ID =
  'CBVNNLIKGYDXDP4KFXIB4K7VDFDZFDG74UJHNMQTO3XAA65DBSTPPJCP';

export const explorerContract = (id: string) =>
  `https://stellar.expert/explorer/testnet/contract/${id}`;
export const explorerTx = (hash: string) =>
  `https://stellar.expert/explorer/testnet/tx/${hash}`;

/** Prompt Freighter for access and return the user's public key (G...). */
export async function connectWallet(): Promise<string> {
  const res: any = await requestAccess();
  if (res?.error) throw new Error(String(res.error));
  const addr: string | undefined = res?.address || ((await getAddress()) as any)?.address;
  if (!addr) throw new Error('Freighter did not return an address. Is it unlocked?');
  return addr;
}

// Adapter matching the SDK's expected signTransaction signature, backed by Freighter.
const signTransaction = async (
  xdr: string,
  opts?: { networkPassphrase?: string; address?: string },
) => {
  const res: any = await freighterSign(xdr, {
    networkPassphrase: opts?.networkPassphrase || NETWORK_PASSPHRASE,
    address: opts?.address,
  });
  if (res?.error) throw new Error(String(res.error));
  return { signedTxXdr: res.signedTxXdr as string, signerAddress: res.signerAddress as string };
};

/** Build a contract client bound to the given contract id and signer. */
export async function makeClient(contractId: string, publicKey?: string): Promise<any> {
  const opts: any = {
    contractId,
    networkPassphrase: NETWORK_PASSPHRASE,
    rpcUrl: RPC_URL,
    publicKey,
    signTransaction,
  };
  return await StellarSdk.contract.Client.from(opts);
}

/** Read-only call: simulate and return the decoded result (no signing). */
export async function readCall(client: any, method: string, args?: any): Promise<any> {
  const at = args === undefined ? await client[method]() : await client[method](args);
  return at.result;
}

/** State-changing call: simulate, sign with Freighter, submit; return value + tx hash. */
export async function writeCall(
  client: any,
  method: string,
  args?: any,
): Promise<{ value: any; hash?: string }> {
  const at = args === undefined ? await client[method]() : await client[method](args);
  const sent = await at.signAndSend();
  let value = sent.result;
  // Contract fns returning Result<T, Error> come back as a Result wrapper — unwrap it
  // (throws the contract error if it failed).
  if (value && typeof value === 'object' && typeof value.unwrap === 'function') {
    value = value.unwrap();
  }
  const hash: string | undefined =
    sent?.sendTransactionResponse?.hash ?? sent?.getTransactionResponse?.txHash;
  return { value, hash };
}

/** Convert a raw form string into the JS value the SDK expects for an arg type. */
export function convertArg(type: ArgType, raw: string): any {
  const v = (raw ?? '').trim();
  switch (type) {
    case 'address':
    case 'token':
      return v;
    case 'i128':
    case 'u64':
      return BigInt(v || '0');
    case 'u32':
      return Number(v || '0');
    case 'vec_i128':
      return v
        .split(',')
        .map((s) => s.trim())
        .filter(Boolean)
        .map((s) => BigInt(s));
  }
}

/** Decode a Status field (stored as an integer enum 0/1/2) defensively. */
export function statusLabel(s: any): string {
  if (s === null || s === undefined) return '—';
  if (typeof s === 'number') return ['Pending', 'Paid', 'Cancelled'][s] ?? String(s);
  if (typeof s === 'bigint') return ['Pending', 'Paid', 'Cancelled'][Number(s)] ?? String(s);
  if (typeof s === 'string') return s;
  if (typeof s === 'object' && s.tag) return String(s.tag);
  return String(s);
}
