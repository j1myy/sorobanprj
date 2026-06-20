import { useState } from 'react';
import type { FnDef } from './contracts';
import {
  makeClient,
  readCall,
  writeCall,
  convertArg,
  statusLabel,
  explorerTx,
} from './stellar';

export type LogEntry = { kind: 'ok' | 'err' | 'info'; text: string; hash?: string };

/** Recursively render a decoded contract result (handles BigInt, structs, enums). */
function ResultView({ value, k }: { value: any; k?: string }) {
  if (value === undefined || value === null) return <span className="muted">∅ none</span>;
  if (k === 'status') return <span className="pill ok">{statusLabel(value)}</span>;
  if (typeof value === 'bigint') return <span className="mono">{value.toString()}</span>;
  if (typeof value === 'boolean') return <span>{value ? 'true' : 'false'}</span>;
  if (typeof value === 'string') {
    const short = value.length > 16 ? `${value.slice(0, 6)}…${value.slice(-6)}` : value;
    return <span className="mono" title={value}>{short}</span>;
  }
  if (typeof value === 'number') return <span className="mono">{String(value)}</span>;
  if (Array.isArray(value)) {
    return <span className="mono">[{value.map((x) => String(x)).join(', ')}]</span>;
  }
  if (typeof value === 'object') {
    if (value.tag) return <span>{String(value.tag)}</span>;
    return (
      <dl className="kv">
        {Object.entries(value).map(([key, v]) => (
          <div className="kvrow" key={key}>
            <dt>{key}</dt>
            <dd><ResultView value={v} k={key} /></dd>
          </div>
        ))}
      </dl>
    );
  }
  return <span>{String(value)}</span>;
}

export function InvokeCard({
  contractId,
  address,
  fn,
  onLog,
}: {
  contractId: string;
  address: string;
  fn: FnDef;
  onLog: (e: LogEntry) => void;
}) {
  const [vals, setVals] = useState<Record<string, string>>(
    Object.fromEntries(fn.args.map((a) => [a.name, ''])),
  );
  const [busy, setBusy] = useState(false);
  const [done, setDone] = useState(false);
  const [result, setResult] = useState<any>(undefined);
  const [hash, setHash] = useState<string | undefined>(undefined);

  const ready = Boolean(contractId) && Boolean(address);
  const set = (name: string, v: string) => setVals((s) => ({ ...s, [name]: v }));

  async function submit() {
    setBusy(true);
    setDone(false);
    setHash(undefined);
    try {
      const argObj: any = {};
      for (const a of fn.args) argObj[a.name] = convertArg(a.type, vals[a.name] ?? '');
      const client = await makeClient(contractId, address);
      const hasArgs = fn.args.length > 0;

      if (fn.kind === 'read') {
        const v = await readCall(client, fn.name, hasArgs ? argObj : undefined);
        setResult(v);
        setDone(true);
        onLog({ kind: 'info', text: `${fn.name} → read ok` });
      } else {
        const { value, hash: h } = await writeCall(client, fn.name, hasArgs ? argObj : undefined);
        setResult(value);
        setHash(h);
        setDone(true);
        onLog({ kind: 'ok', text: `${fn.name} ✓${value !== undefined ? ` → ${String(value)}` : ''}`, hash: h });
      }
    } catch (err: any) {
      onLog({ kind: 'err', text: `${fn.name} failed: ${err?.message ?? String(err)}` });
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className={`fn ${fn.kind}`}>
      <div className="fn-head">
        <code className="fn-name">{fn.name}</code>
        <span className={`kindtag ${fn.kind}`}>{fn.kind}</span>
      </div>
      {fn.desc && <p className="hint">{fn.desc}</p>}

      {fn.args.map((a) => (
        <div className="argrow" key={a.name}>
          <label>{a.name} <span className="muted">· {a.type}</span></label>
          <div className="argline">
            <input
              value={vals[a.name] ?? ''}
              placeholder={a.placeholder ?? a.type}
              spellCheck={false}
              onChange={(e) => set(a.name, e.target.value)}
            />
            {a.type === 'address' && address && (
              <button className="chip" type="button" onClick={() => set(a.name, address)}>me</button>
            )}
          </div>
        </div>
      ))}

      <button
        className={`btn ${fn.kind === 'write' ? 'primary' : ''}`}
        disabled={busy || !ready}
        onClick={submit}
      >
        {busy ? 'Running…' : !ready ? 'Connect + set id' : fn.kind === 'read' ? 'Call' : 'Send'}
      </button>

      {done && (
        <div className="result">
          {result === undefined ? (
            <span className="muted">{fn.kind === 'write' ? '✓ submitted' : '∅ none'}</span>
          ) : (
            <ResultView value={result} />
          )}
          {hash && (
            <a className="link" href={explorerTx(hash)} target="_blank" rel="noreferrer"> tx ↗</a>
          )}
        </div>
      )}
    </div>
  );
}
