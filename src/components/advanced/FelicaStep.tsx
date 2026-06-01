import { useAdvanced } from '../../hooks/useAdvanced';
import { useState } from 'react';

export function FelicaStep({ advanced }: { advanced: ReturnType<typeof useAdvanced> }) {
  const [info, setInfo] = useState<null | { idm: string; pmm: string }>(null);
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
      <h3>Felica Info</h3>
      <button onClick={fetch} disabled={loading}>
        {loading ? 'Querying…' : 'Read Felica Tag'}
      </button>

      {error && (
        <div style={{ color: 'var(--red-bright)', marginTop: '12px' }}>
          Error: {error}
        </div>
      )}

      {info && (
        <div style={{ marginTop: '16px', fontFamily: 'var(--font-mono)' }}>
          <div>IDm: {info.idm}</div>
          <div>PMm: {info.pmm}</div>
        </div>
      )}
    </div>
  );
}