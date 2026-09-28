import { useAdvanced } from '../../hooks/useAdvanced';
import { useState } from 'react';

export function MadStep({ advanced }: { advanced: ReturnType<typeof useAdvanced> }) {
  const [data, setData] = useState<string>('');
  const [output, setOutput] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

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

  return (
    <div style={{ maxWidth: '600px' }}>
      <h3>MAD (Mifare Application Directory)</h3>
      <p style={{ color: 'var(--text-secondary)', fontSize: '13px', marginBottom: '12px' }}>
        MAD v4.23346: read / write / verify / decode / encode
      </p>

      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px', marginBottom: '12px' }}>
        <button onClick={() => run(advanced.madRead, 'read')} disabled={loading}>Read</button>
        <button onClick={() => run(advanced.madVerify, 'verify')} disabled={loading}>Verify</button>
        <button onClick={() => run(advanced.madDecode, 'decode')} disabled={loading}>Decode</button>
      </div>

      <div style={{ marginBottom: '12px', padding: '10px', background: 'var(--bg-tertiary)', borderRadius: '6px' }}>
        <label htmlFor='mad-data' style={{ display: 'block', marginBottom: '4px', fontSize: '13px' }}>
          Data (for write / encode):
        </label>
        <input id='mad-data' type='text' value={data} onChange={(e) => setData(e.target.value)}
          placeholder='e.g. 0001020304' style={{ width: '100%', padding: '4px' }} />
        <div style={{ display: 'flex', gap: '8px', marginTop: '6px' }}>
          <button onClick={() => run(() => advanced.madWrite(data), 'write')} disabled={loading || !data}>Write</button>
          <button onClick={() => run(() => advanced.madEncode(data), 'encode')} disabled={loading || !data}>Encode</button>
        </div>
      </div>

      {error && (<div style={{ color: 'var(--red-bright)', marginTop: '12px' }}>Error: {error}</div>)}
      {output && (
        <div style={{ marginTop: '16px', fontFamily: 'var(--font-mono)', whiteSpace: 'pre-wrap', background: 'var(--bg-tertiary)', padding: '12px', borderRadius: '6px' }}>
          {output}
        </div>
      )}
    </div>
  );
}