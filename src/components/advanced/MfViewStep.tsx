import { useAdvanced } from '../../hooks/useAdvanced';
import { useState } from 'react';
import { DumpFilePicker } from './DumpFilePicker';

export function MfViewStep({ advanced }: { advanced: ReturnType<typeof useAdvanced> }) {
  const [file, setFile] = useState<string>('');
  const [output, setOutput] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  // Increment this to force DumpFilePicker to re-scan after a dump/autopwn
  const [refreshKey, setRefreshKey] = useState(0);

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
      // After a dump or autopwn, force the file list to re-scan so the
      // new dump file appears in the dropdown automatically.
      if (label === 'dump' || label === 'autopwn') {
        setRefreshKey((k) => k + 1);
      }
    } catch (e: any) {
      setError(getErrorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ maxWidth: '640px' }}>
      <h3>Mifare View (RKF / VIGIK / HID PACS)</h3>
      <p style={{ color: 'var(--text-secondary)', fontSize: '13px', marginBottom: '12px' }}>
        Decode MIFARE Classic dump files (RKF travel cards, VIGIK access, HID PACS).
        Dump a card first using the buttons below, or use the wizard scan flow.
      </p>

      {/* Read / Write / Break / Emulate section */}
      <div style={{ marginBottom: '12px', padding: '10px', background: 'var(--bg-tertiary)', borderRadius: '6px' }}>
        <div style={{ fontSize: '13px', marginBottom: '6px', color: 'var(--green-dim)' }}>Read / Write / Break / Emulate</div>
        <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap', marginBottom: '6px' }}>
          <button onClick={() => run(advanced.hfMfDump, 'dump')} disabled={loading}
            style={{ background: 'var(--bg-void)', fontFamily: 'var(--font-mono)', fontSize: '12px', padding: '4px 12px', cursor: 'pointer', color: 'var(--green-bright)', border: '1px solid var(--green-bright)' }}>
            Dump
          </button>
          <button onClick={() => run(advanced.hfMfCview, 'cview')} disabled={loading}
            style={{ background: 'var(--bg-void)', fontFamily: 'var(--font-mono)', fontSize: '12px', padding: '4px 12px', cursor: 'pointer', color: 'var(--green-mid)', border: '1px solid var(--green-mid)' }}>
            CView
          </button>
          <button onClick={() => run(advanced.hfMfAutopwn, 'autopwn')} disabled={loading}
            style={{ background: 'var(--bg-void)', fontFamily: 'var(--font-mono)', fontSize: '12px', padding: '4px 12px', cursor: 'pointer', color: 'var(--amber)', border: '1px solid var(--amber)' }}>
            Autopwn
          </button>
          <button onClick={() => run(() => advanced.mfSim(), 'sim')} disabled={loading}
            style={{ background: 'var(--bg-void)', fontFamily: 'var(--font-mono)', fontSize: '12px', padding: '4px 12px', cursor: 'pointer', color: 'var(--green-mid)', border: '1px solid var(--green-mid)' }}>
            Sim
          </button>
          <button onClick={() => run(() => advanced.mfEclr(), 'eclr')} disabled={loading}
            style={{ background: 'var(--bg-void)', fontFamily: 'var(--font-mono)', fontSize: '12px', padding: '4px 12px', cursor: 'pointer', color: 'var(--green-dim)', border: '1px solid var(--green-dim)' }}>
            ECLR
          </button>
        </div>
        <div style={{ color: 'var(--green-dim)', fontSize: '11px' }}>
          Dump = hf mf dump &nbsp;|&nbsp; CView = hf mf cview (magic backdoor) &nbsp;|&nbsp; Autopwn = hf mf autopwn &nbsp;|&nbsp; Sim = hf mf sim &nbsp;|&nbsp; ECLR = hf mf eclr
        </div>
      </div>

      {/* View a dump file section */}
      <div style={{ marginBottom: '12px', padding: '10px', background: 'var(--bg-tertiary)', borderRadius: '6px' }}>
        <div style={{ fontSize: '13px', marginBottom: '6px', color: 'var(--green-dim)' }}>2. View a dump file</div>
        <DumpFilePicker
          advanced={advanced}
          value={file}
          onChange={setFile}
          disabled={loading}
          placeholder='e.g. hf-mf-0CCBF0CF-dump.bin'
          triggerRefresh={refreshKey}
        />
        <button onClick={() => run(() => advanced.hfMfView(file), 'view')} disabled={loading || !file}
          style={{ marginTop: '6px', background: 'var(--bg-void)', fontFamily: 'var(--font-mono)', fontSize: '13px', fontWeight: 600, padding: '4px 16px', cursor: (loading || !file) ? 'default' : 'pointer', color: 'var(--green-bright)', border: '2px solid var(--green-bright)' }}>
          View dump
        </button>
      </div>

      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px', marginBottom: '12px' }}>
        <button onClick={() => run(advanced.hfMfViewSelftest, 'view --selftest')} disabled={loading}>Selftest</button>
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