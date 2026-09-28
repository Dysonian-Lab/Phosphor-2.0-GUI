import { useAdvanced } from '../../hooks/useAdvanced';
import { useState } from 'react';

export function CalypsoStep({ advanced }: { advanced: ReturnType<typeof useAdvanced> }) {
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
      <h3>Calypso</h3>
      <p style={{ color: 'var(--text-secondary)', fontSize: '13px', marginBottom: '12px' }}>
        Calypso v4.23346: info / dump / list
      </p>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
        <button onClick={() => run(advanced.calypsoInfo, 'info')} disabled={loading}>Info</button>
        <button onClick={() => run(advanced.calypsoDump, 'dump')} disabled={loading}>Dump</button>
        <button onClick={() => run(advanced.calypsoList, 'list')} disabled={loading}>List</button>
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