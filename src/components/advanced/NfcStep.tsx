import { useAdvanced } from '../../hooks/useAdvanced';
import { useState } from 'react';

export function NfcStep({ advanced }: { advanced: ReturnType<typeof useAdvanced> }) {
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
      <h3>NFC Encode</h3>
      <p style={{ color: 'var(--text-secondary)', fontSize: '13px', marginBottom: '12px' }}>
        NFC v4.23346: encode data to NFC Forum format
      </p>
      <div style={{ padding: '10px', background: 'var(--bg-tertiary)', borderRadius: '6px' }}>
        <label htmlFor='nfc-data' style={{ display: 'block', marginBottom: '4px', fontSize: '13px' }}>
          Data to encode:
        </label>
        <input id='nfc-data' type='text' value={data} onChange={(e) => setData(e.target.value)}
          placeholder='e.g. Hello or hex 48656c6c6f' style={{ width: '100%', padding: '4px' }} />
        <button onClick={() => run(() => advanced.nfcEncode(data), 'encode')} disabled={loading || !data}
          style={{ marginTop: '6px' }}>Encode</button>
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