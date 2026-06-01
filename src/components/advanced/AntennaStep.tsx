import { useAdvanced } from '../../hooks/useAdvanced';
import { useState } from 'react';

export function AntennaStep({ advanced }: { advanced: ReturnType<typeof useAdvanced> }) {
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

  const run = async () => {
    setLoading(true);
    setError(null);
    setOutput(null);
    try {
      const result = await advanced.hwMeasure();
      setOutput(result.output);
    } catch (e: any) {
      setError(getErrorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ maxWidth: '600px' }}>
      <h3>Antenna Measurement</h3>
      <button onClick={run} disabled={loading}>
        {loading ? 'Measuring…' : 'Run HW Measure'}
      </button>

      {error && (
        <div style={{ color: 'var(--red-bright)', marginTop: '12px' }}>
          Error: {error}
        </div>
      )}

      {output && (
        <div style={{ marginTop: '16px', fontFamily: 'var(--font-mono)', whiteSpace: 'pre-wrap' }}>
          <strong>HW Measure Output:</strong><br />
          {output}
        </div>
      )}
    </div>
  );
}