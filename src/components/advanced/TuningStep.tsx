import { useAdvanced } from '../../hooks/useAdvanced';
import { useState } from 'react';

export function TuningStep({ advanced }: { advanced: ReturnType<typeof useAdvanced> }) {
  const [hwOutput, setHwOutput] = useState<string | null>(null);
  const [lfOutput, setLfOutput] = useState<string | null>(null);
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

  const runHwTune = async () => {
    setLoading(true);
    setError(null);
    setHwOutput(null);
    try {
      const output = await advanced.hwTune();
      setHwOutput(output);
    } catch (e: any) {
      setError(getErrorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  const runLfTune = async () => {
    setLoading(true);
    setError(null);
    setLfOutput(null);
    try {
      const output = await advanced.lfTune();
      setLfOutput(output);
    } catch (e: any) {
      setError(getErrorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ maxWidth: '600px' }}>
      <h3>Hardware Tuning</h3>
      <div style={{ marginBottom: '16px' }}>
        <button onClick={runHwTune} disabled={loading}>
          {loading ? 'Running…' : 'Run HW Tune'}
        </button>
        {hwOutput && (
          <div style={{ marginTop: '8px', fontFamily: 'var(--font-mono)', whiteSpace: 'pre-wrap' }}>
            <strong>HW Tune Output:</strong><br />
            {hwOutput}
          </div>
        )}
      </div>

      <div>
        <button onClick={runLfTune} disabled={loading}>
          {loading ? 'Applying…' : 'Apply LF Tune'}
        </button>
        {lfOutput && (
          <div style={{ marginTop: '8px', fontFamily: 'var(--font-mono)', whiteSpace: 'pre-wrap' }}>
            <strong>LF Tune Output:</strong><br />
            {lfOutput}
          </div>
        )}
      </div>

      {error && (
        <div style={{ color: 'var(--red-bright)', marginTop: '12px' }}>
          Error: {error}
        </div>
      )}
    </div>
  );
}