import { useAdvanced } from '../../hooks/useAdvanced';
import { useState } from 'react';

export function LegicStep({ advanced }: { advanced: ReturnType<typeof useAdvanced> }) {
  const [info, setInfo] = useState<null | { uid: string; atqa: string }>(null);
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
      const data = await advanced.legicInfo();
      setInfo({ uid: data.uid, atqa: data.atqa });
    } catch (e: any) {
      setError(getErrorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ maxWidth: '600px' }}>
      <h3>LEGIC Info</h3>
      <button onClick={fetch} disabled={loading}>
        {loading ? 'Querying…' : 'Read LEGIC Tag'}
      </button>

      {error && (
        <div style={{ color: 'var(--red-bright)', marginTop: '12px' }}>
          Error: {error}
        </div>
      )}

      {info && (
        <div style={{ marginTop: '16px', fontFamily: 'var(--font-mono)' }}>
          <div>UID: {info.uid}</div>
          <div>ATQA: {info.atqa}</div>
        </div>
      )}
    </div>
  );
}