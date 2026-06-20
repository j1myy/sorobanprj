import { useMemo, useState } from 'react';
import {
  connectWallet,
  makeClient,
  readCall,
  writeCall,
  statusLabel,
  explorerContract,
  explorerTx,
  DEFAULT_CONTRACT_ID,
} from './stellar';

type LogEntry = { kind: 'ok' | 'err' | 'info'; text: string; hash?: string };

// One XLM = 10,000,000 base units (7 decimals). Handy for the demo.
const XLM = 10_000_000n;

export default function App() {
  const [contractId, setContractId] = useState(DEFAULT_CONTRACT_ID);
  const [address, setAddress] = useState<string>('');
  const [busy, setBusy] = useState<string>(''); // label of the in-flight action
  const [log, setLog] = useState<LogEntry[]>([]);

  // form state
  const [token, setToken] = useState('');
  const [exporter, setExporter] = useState('');
  const [buyer, setBuyer] = useState('');
  const [amount, setAmount] = useState('10000000');
  const [payId, setPayId] = useState('1');
  const [viewId, setViewId] = useState('1');
  const [invoice, setInvoice] = useState<any | null>(null);
  const [totalSettled, setTotalSettled] = useState<string>('');

  const connected = Boolean(address);
  const short = (a: string) => (a ? `${a.slice(0, 6)}…${a.slice(-6)}` : '');

  const pushLog = (e: LogEntry) => setLog((l) => [e, ...l].slice(0, 30));

  // Build a client lazily for each action so it always uses the latest id/signer.
  const client = useMemo(() => null, []); // placeholder to keep deps explicit
  void client;

  async function run<T>(label: string, fn: (client: any) => Promise<T>) {
    if (!connected) {
      pushLog({ kind: 'err', text: 'Connect Freighter first.' });
      return;
    }
    setBusy(label);
    try {
      const c = await makeClient(contractId, address);
      const result = await fn(c);
      return result;
    } catch (err: any) {
      pushLog({ kind: 'err', text: `${label} failed: ${err?.message ?? String(err)}` });
      return undefined;
    } finally {
      setBusy('');
    }
  }

  async function onConnect() {
    try {
      const a = await connectWallet();
      setAddress(a);
      if (!exporter) setExporter(a);
      if (!buyer) setBuyer(a);
      pushLog({ kind: 'info', text: `Connected ${short(a)}` });
    } catch (err: any) {
      pushLog({ kind: 'err', text: `Connect failed: ${err?.message ?? String(err)}` });
    }
  }

  async function onInitialize() {
    if (!token.trim()) {
      pushLog({ kind: 'err', text: 'Enter a token contract id (the USDC/XLM SAC).' });
      return;
    }
    const r = await run('initialize', (c) => writeCall(c, 'initialize', { token: token.trim() }));
    if (r) pushLog({ kind: 'ok', text: 'Contract initialized with token.', hash: r.hash });
  }

  async function onIssue() {
    const r = await run('issue_invoice', (c) =>
      writeCall(c, 'issue_invoice', {
        exporter: exporter.trim(),
        buyer: buyer.trim(),
        amount: BigInt(amount || '0'),
      }),
    );
    if (r) {
      const id = String(r.value);
      setPayId(id);
      setViewId(id);
      pushLog({ kind: 'ok', text: `Invoice #${id} issued.`, hash: r.hash });
    }
  }

  async function onPay() {
    const r = await run('pay_invoice', (c) =>
      writeCall(c, 'pay_invoice', { id: BigInt(payId || '0') }),
    );
    if (r) {
      pushLog({ kind: 'ok', text: `Invoice #${payId} paid — settled to exporter.`, hash: r.hash });
      await onView(payId);
    }
  }

  async function onView(idArg?: string) {
    const id = idArg ?? viewId;
    const inv = await run('get_invoice', (c) => readCall(c, 'get_invoice', { id: BigInt(id || '0') }));
    if (inv === undefined) return;
    setInvoice(inv ?? null);
    if (!inv) pushLog({ kind: 'info', text: `No invoice #${id} found.` });
    const total = await run('total_settled', (c) => readCall(c, 'total_settled'));
    if (total !== undefined) setTotalSettled(String(total));
  }

  return (
    <div className="page">
      <header className="topbar">
        <div className="brand">
          <span className="logo">🌉</span>
          <div>
            <h1>PesoBridge</h1>
            <p className="sub">Instant cross-border B2B invoice settlement · Stellar testnet</p>
          </div>
        </div>
        <div className="wallet">
          {connected ? (
            <span className="pill ok" title={address}>● {short(address)}</span>
          ) : (
            <button className="btn primary" onClick={onConnect}>Connect Freighter</button>
          )}
        </div>
      </header>

      <div className="config">
        <label>Contract ID</label>
        <input value={contractId} onChange={(e) => setContractId(e.target.value)} spellCheck={false} />
        <a className="link" href={explorerContract(contractId)} target="_blank" rel="noreferrer">
          View on stellar.expert ↗
        </a>
      </div>

      <main className="grid">
        {/* Admin setup */}
        <section className="card">
          <h2>1 · Initialize <span className="tag">admin, once</span></h2>
          <p className="hint">
            Set which token gets paid. For a demo, use the native XLM Stellar Asset Contract id
            (<code>stellar contract id asset --asset native --network testnet</code>).
          </p>
          <label>Token contract id</label>
          <input placeholder="C…" value={token} onChange={(e) => setToken(e.target.value)} spellCheck={false} />
          <button className="btn" disabled={busy === 'initialize'} onClick={onInitialize}>
            {busy === 'initialize' ? 'Initializing…' : 'Initialize'}
          </button>
        </section>

        {/* Issue */}
        <section className="card">
          <h2>2 · Issue invoice <span className="tag">exporter</span></h2>
          <label>Exporter (receives funds)</label>
          <input value={exporter} onChange={(e) => setExporter(e.target.value)} spellCheck={false} />
          <label>Buyer (will pay)</label>
          <input value={buyer} onChange={(e) => setBuyer(e.target.value)} spellCheck={false} />
          <label>Amount <span className="muted">(base units · {amount && `${(Number(BigInt(amount || '0') ) / Number(XLM)).toString()} XLM`})</span></label>
          <input value={amount} onChange={(e) => setAmount(e.target.value.replace(/[^0-9]/g, ''))} inputMode="numeric" />
          <button className="btn" disabled={busy === 'issue_invoice'} onClick={onIssue}>
            {busy === 'issue_invoice' ? 'Issuing…' : 'Issue invoice'}
          </button>
        </section>

        {/* Pay */}
        <section className="card">
          <h2>3 · Pay invoice <span className="tag">buyer signs</span></h2>
          <p className="hint">Freighter must be on the buyer account to authorize the payment.</p>
          <label>Invoice id</label>
          <input value={payId} onChange={(e) => setPayId(e.target.value.replace(/[^0-9]/g, ''))} inputMode="numeric" />
          <button className="btn primary" disabled={busy === 'pay_invoice'} onClick={onPay}>
            {busy === 'pay_invoice' ? 'Paying…' : 'Pay invoice'}
          </button>
        </section>

        {/* View */}
        <section className="card">
          <h2>4 · Invoice status</h2>
          <div className="row">
            <input value={viewId} onChange={(e) => setViewId(e.target.value.replace(/[^0-9]/g, ''))} inputMode="numeric" />
            <button className="btn" disabled={busy === 'get_invoice'} onClick={() => onView()}>
              {busy === 'get_invoice' ? 'Loading…' : 'Fetch'}
            </button>
          </div>
          {invoice ? (
            <dl className="kv">
              <dt>Status</dt><dd><span className={`pill ${statusLabel(invoice.status) === 'Paid' ? 'ok' : ''}`}>{statusLabel(invoice.status)}</span></dd>
              <dt>Amount</dt><dd>{String(invoice.amount)}</dd>
              <dt>Exporter</dt><dd className="mono">{short(String(invoice.exporter))}</dd>
              <dt>Buyer</dt><dd className="mono">{short(String(invoice.buyer))}</dd>
              <dt>Paid at</dt><dd>{String(invoice.paid_at) === '0' ? '—' : String(invoice.paid_at)}</dd>
            </dl>
          ) : (
            <p className="muted">No invoice loaded.</p>
          )}
          {totalSettled !== '' && (
            <p className="total">Total settled by contract: <b>{totalSettled}</b></p>
          )}
        </section>
      </main>

      <section className="logbox">
        <h2>Activity</h2>
        {log.length === 0 ? (
          <p className="muted">Actions and transaction results show up here.</p>
        ) : (
          <ul>
            {log.map((e, i) => (
              <li key={i} className={e.kind}>
                <span>{e.text}</span>
                {e.hash && (
                  <a className="link" href={explorerTx(e.hash)} target="_blank" rel="noreferrer"> tx ↗</a>
                )}
              </li>
            ))}
          </ul>
        )}
      </section>

      <footer className="foot">
        Demo dApp · pays via USDC/XLM on Stellar · <a className="link" href="https://github.com/j1myy/sorobanprj" target="_blank" rel="noreferrer">source</a>
      </footer>
    </div>
  );
}
