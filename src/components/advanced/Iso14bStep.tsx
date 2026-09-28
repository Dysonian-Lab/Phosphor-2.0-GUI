import { useAdvanced } from '../../hooks/useAdvanced';
import { useState } from 'react';
import { DumpFilePicker } from './DumpFilePicker';

export function Iso14bStep({ advanced }: { advanced: ReturnType<typeof useAdvanced> }) {
  const [info, setInfo] = useState<null | { uid: string; atqa: string }>(null);
  const [output, setOutput] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [block, setBlock] = useState<string>('0');
  const [viewFile, setViewFile] = useState<string>('');

  const getErrorMessage = (e: any): string => {
    if (typeof e === 'string') return e;
    if (e && typeof e === 'object') {
      const val = Object.values(e)[0];
      return typeof val === 'string' ? val : JSON.stringify(e);
    }
    return String(e);
  };

  const run = async (fn: () => Promise<string>, label: string) => {
    setLoading(true);
    setError(null);
    setOutput(null);
    try {
      const result = await fn();
      setOutput(`=== ${label} ===\n${result}`);
    } catch (e: any) {
      setError(getErrorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  const fetch = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await advanced.iso14bInfo();
      setInfo({ uid: data.uid, atqa: data.atqa });
    } catch (e: any) {
      setError(getErrorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ maxWidth: '600px' }}>
      <h3>ISO 14443‑B</h3>
      <div style={{ marginBottom: '12px' }}>
        <button onClick={fetch} disabled={loading}>
          {loading ? 'Querying…' : 'Read ISO 14443‑B Tag'}
        </button>
      </div>

      {/* v4.23346: rdbl / ctrdbl / view --selftest / view -f */}
      <div style={{ marginBottom: '12px', padding: '10px', background: 'var(--bg-tertiary)', borderRadius: '6px' }}>
        <label htmlFor='14b-block' style={{ display: 'block', marginBottom: '4px', fontSize: '13px' }}>
          Block number:
        </label>
        <input id='14b-block' type='text' value={block} onChange={(e) => setBlock(e.target.value)}
          placeholder='0' style={{ width: '80px', padding: '4px' }} />
        <div style={{ display: 'flex', gap: '8px', marginTop: '6px' }}>
          <button onClick={() => run(() => advanced.iso14bRdbl(Number(block)), 'rdbl')} disabled={loading}>Read block</button>
          <button onClick={() => run(() => advanced.iso14bCtrdbl(Number(block)), 'ctrdbl')} disabled={loading}>CTR read</button>
          <button onClick={() => run(advanced.iso14bViewSelftest, 'view --selftest')} disabled={loading}>View selftest</button>
        </div>
      </div>

      {/* v4.23346: hf 14b view -f (MyKey/COGES decode) */}
      <div style={{ marginBottom: '12px', padding: '10px', background: 'var(--bg-tertiary)', borderRadius: '6px' }}>
        <label htmlFor='14b-view-file' style={{ display: 'block', marginBottom: '4px', fontSize: '13px' }}>
          Dump file (MyKey/COGES decode):
        </label>
        <DumpFilePicker
          advanced={advanced}
          value={viewFile}
          onChange={setViewFile}
          disabled={loading}
          placeholder='e.g. hf-14b-D0021F673CB26556-dump.json'
        />
        <button onClick={() => run(() => advanced.iso14bView(viewFile), 'view')} disabled={loading || !viewFile}
          style={{ marginTop: '6px' }}>View dump</button>
      </div>

      {error && (<div style={{ color: 'var(--red-bright)', marginTop: '12px' }}>Error: {error}</div>)}
      {info && (
        <div style={{ marginTop: '16px', fontFamily: 'var(--font-mono)' }}>
          <div>UID: {info.uid}</div>
          <div>ATQA: {info.atqa}</div>
        </div>
      )}
      {output && (
        <div style={{ marginTop: '16px', fontFamily: 'var(--font-mono)', whiteSpace: 'pre-wrap', background: 'var(--bg-tertiary)', padding: '12px', borderRadius: '6px' }}>
          {output}
        </div>
      )}
    </div>
  );
}