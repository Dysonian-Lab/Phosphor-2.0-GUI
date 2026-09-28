import { useAdvanced } from '../../hooks/useAdvanced';
import { useState } from 'react';

export function FelicaStep({ advanced }: { advanced: ReturnType<typeof useAdvanced> }) {
  const [info, setInfo] = useState<null | { idm: string; pmm: string }>(null);
  const [output, setOutput] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dumpPath, setDumpPath] = useState<string>('');

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
      const data = await advanced.felicaInfo();
      setInfo({ idm: data.idm, pmm: data.pmm });
    } catch (e: any) {
      setError(getErrorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ maxWidth: '600px' }}>
      <h3>Felica</h3>
      <div style={{ marginBottom: '12px' }}>
        <button onClick={fetch} disabled={loading}>
          {loading ? 'Querying…' : 'Read Felica Tag'}
        </button>
      </div>

      {/* v4.23346: sim from dump file */}
      <div style={{ marginBottom: '12px', padding: '10px', background: 'var(--bg-tertiary)', borderRadius: '6px' }}>
        <label htmlFor='felica-dump' style={{ display: 'block', marginBottom: '4px', fontSize: '13px' }}>
          Dump path (for sim):
        </label>
        <input id='felica-dump' type='text' value={dumpPath} onChange={(e) => setDumpPath(e.target.value)}
          placeholder='e.g. C:/dumps/felica.dump' style={{ width: '100%', padding: '4px' }} />
        <button onClick={() => run(() => advanced.felicaSim(dumpPath), 'sim')} disabled={loading || !dumpPath}
          style={{ marginTop: '6px' }}>Simulate from dump</button>
      </div>

      {error && (<div style={{ color: 'var(--red-bright)', marginTop: '12px' }}>Error: {error}</div>)}
      {info && (
        <div style={{ marginTop: '16px', fontFamily: 'var(--font-mono)' }}>
          <div>IDm: {info.idm}</div>
          <div>PMm: {info.pmm}</div>
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