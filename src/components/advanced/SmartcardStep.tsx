import { useAdvanced } from '../../hooks/useAdvanced';
import { useState } from 'react';

export function SmartcardStep({ advanced }: { advanced: ReturnType<typeof useAdvanced> }) {
  const [output, setOutput] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [ta1, setTa1] = useState<string>('');

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
      <h3>Smart Card (ISO 7816-3)</h3>
      <p style={{ color: 'var(--text-secondary)', fontSize: '13px', marginBottom: '12px' }}>
        Smart card v4.23346: PPS — Protocol Parameter Selection (RDV4 module required)
      </p>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px', marginBottom: '12px' }}>
        <button onClick={() => run(advanced.smartPps, 'pps')} disabled={loading}>PPS</button>
        <button onClick={() => run(advanced.smartPpsT0, 'pps --t0')} disabled={loading}>T=0</button>
        <button onClick={() => run(advanced.smartPpsT1, 'pps --t1')} disabled={loading}>T=1</button>
      </div>
      <div style={{ padding: '10px', background: 'var(--bg-tertiary)', borderRadius: '6px' }}>
        <label htmlFor='smart-ta1' style={{ display: 'block', marginBottom: '4px', fontSize: '13px' }}>
          TA1 byte (hex):
        </label>
        <input id='smart-ta1' type='text' value={ta1} onChange={(e) => setTa1(e.target.value)}
          placeholder='e.g. 93' style={{ width: '80px', padding: '4px' }} />
        <button onClick={() => run(() => advanced.smartPpsTa1(ta1), 'pps --ta1')} disabled={loading || !ta1}
          style={{ marginTop: '6px' }}>Negotiate TA1</button>
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