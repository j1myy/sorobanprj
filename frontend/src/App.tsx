import { useEffect, useMemo, useState } from 'react';
import { CONTRACTS } from './contracts';
import { InvokeCard, type LogEntry } from './InvokeCard';
import { connectWallet, explorerContract } from './stellar';

const LS_KEY = 'sorobanprj.contractIds';

export default function App() {
  const [address, setAddress] = useState('');
  const [activeKey, setActiveKey] = useState(CONTRACTS[0].key);
  const [log, setLog] = useState<LogEntry[]>([]);

  // Per-contract id overrides, persisted so you don't re-paste each reload.
  const [ids, setIds] = useState<Record<string, string>>(() => {
    const base: Record<string, string> = {};
    for (const c of CONTRACTS) base[c.key] = c.defaultId;
    try {
      const saved = JSON.parse(localStorage.getItem(LS_KEY) || '{}');
      return { ...base, ...saved };
    } catch {
      return base;
    }
  });

  useEffect(() => {
    try {
      localStorage.setItem(LS_KEY, JSON.stringify(ids));
    } catch {
      /* ignore */
    }
  }, [ids]);

  const connected = Boolean(address);
  const short = (a: string) => (a ? `${a.slice(0, 6)}…${a.slice(-6)}` : '');
  const pushLog = (e: LogEntry) => setLog((l) => [e, ...l].slice(0, 40));

  const active = useMemo(
    () => CONTRACTS.find((c) => c.key === activeKey)!,
    [activeKey],
  );
  const activeId = ids[active.key] ?? '';

  async function onConnect() {
    try {
      const a = await connectWallet();
      setAddress(a);
      pushLog({ kind: 'info', text: `Connected ${short(a)}` });
    } catch (err: any) {
      pushLog({ kind: 'err', text: `Connect failed: ${err?.message ?? String(err)}` });
    }
  }

  return (
    <div className="page">
      <header className="topbar">
        <div className="brand">
          <span className="logo">🪙</span>
          <div>
            <h1>Soroban SME Rails</h1>
            <p className="sub">Payroll · Freelancer escrow · Cross-border invoicing — Stellar testnet</p>
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

      <nav className="tabs">
        {CONTRACTS.map((c) => (
          <button
            key={c.key}
            className={`tab ${c.key === activeKey ? 'active' : ''}`}
            onClick={() => setActiveKey(c.key)}
          >
            <span className="tab-emoji">{c.emoji}</span>
            <span>
              <b>{c.name}</b>
              <small>{c.tagline}</small>
            </span>
          </button>
        ))}
      </nav>

      <section className="panel-head">
        <div className="flow">
          <span className="muted">Flow:</span>
          {active.flow.map((f, i) => (
            <span key={f} className="step">
              {i > 0 && <span className="arrow">→</span>}
              <code>{f}</code>
            </span>
          ))}
        </div>
        <div className="idrow">
          <label>Contract ID</label>
          <input
            value={activeId}
            placeholder="Paste this contract's deployed testnet id (C…)"
            spellCheck={false}
            onChange={(e) => setIds((m) => ({ ...m, [active.key]: e.target.value.trim() }))}
          />
          {activeId ? (
            <a className="link" href={explorerContract(activeId)} target="_blank" rel="noreferrer">explorer ↗</a>
          ) : (
            <span className="warn">needs id</span>
          )}
        </div>
        {!activeId && (
          <p className="hint warnbox">
            No id set for {active.name}. Deploy it and paste the contract id above — it's saved
            locally so you only do this once.
          </p>
        )}
      </section>

      <main className="fngrid">
        {active.fns.map((fn) => (
          <InvokeCard
            key={fn.name}
            contractId={activeId}
            address={address}
            fn={fn}
            onLog={pushLog}
          />
        ))}
      </main>

      <section className="logbox">
        <h2>Activity</h2>
        {log.length === 0 ? (
          <p className="muted">Calls and transaction results show up here.</p>
        ) : (
          <ul>
            {log.map((e, i) => (
              <li key={i} className={e.kind}>
                <span>{e.text}</span>
                {e.hash && (
                  <a className="link" href={`https://stellar.expert/explorer/testnet/tx/${e.hash}`} target="_blank" rel="noreferrer"> tx ↗</a>
                )}
              </li>
            ))}
          </ul>
        )}
      </section>

      <footer className="foot">
        Demo dApps on Stellar testnet ·{' '}
        <a className="link" href="https://github.com/j1myy/sorobanprj" target="_blank" rel="noreferrer">source</a>
      </footer>
    </div>
  );
}
