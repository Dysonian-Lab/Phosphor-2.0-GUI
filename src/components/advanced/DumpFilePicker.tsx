import { useState, useEffect, useCallback } from 'react';
import { useAdvanced } from '../../hooks/useAdvanced';

interface Props {
  advanced: ReturnType<typeof useAdvanced>;
  value: string;
  onChange: (name: string) => void;
  disabled?: boolean;
  placeholder?: string;
  /** Set true to force a refresh of the file list (e.g. after a dump operation) */
  triggerRefresh?: number;
}

/**
 * Dump-file picker. Scans the app directory for PM3 dump files and presents
 * them as a dropdown. No manual path entry — the user just picks from what's
 * on disk. A refresh button re-scans after a dump is created.
 */
export function DumpFilePicker({ advanced, value, onChange, disabled, placeholder, triggerRefresh }: Props) {
  const [files, setFiles] = useState<{ name: string; path: string; size: number }[]>([]);

  const loadFiles = useCallback(async () => {
    try {
      const entries = await advanced.listDumpFiles();
      setFiles(
        entries.map((e) => ({ name: e.name, path: e.path, size: e.size_bytes }))
      );
    } catch {
      // listDumpFiles failed — leave existing list intact
    }
  }, [advanced]);

  // Load on mount and whenever triggerRefresh changes
  useEffect(() => {
    loadFiles();
  }, [loadFiles, triggerRefresh]);

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        disabled={disabled}
        style={{
          width: '100%', padding: '4px',
          background: 'var(--bg-void)', color: 'var(--green-bright)',
          fontFamily: 'var(--font-mono)', fontSize: '13px',
          border: '1px solid var(--green-dim)',
        }}
      >
        <option value=''>{placeholder || '— Select a dump file —'}</option>
        {files.map((f) => (
          <option key={f.path} value={f.name}>
            {f.name}  ({formatSize(f.size)})
          </option>
        ))}
      </select>

      <div style={{ marginTop: '6px', display: 'flex', gap: '8px', alignItems: 'center' }}>
        <button
          onClick={loadFiles}
          disabled={disabled}
          title='Re-scan the app directory for dump files'
          style={{
            background: 'var(--bg-void)', fontFamily: 'var(--font-mono)',
            fontSize: '12px', padding: '4px 12px', cursor: 'pointer',
            color: 'var(--green-dim)', border: '1px solid var(--green-dim)',
          }}
        >
          ↻ Refresh
        </button>
      </div>

      {files.length === 0 && (
        <div style={{ color: 'var(--amber)', marginTop: '6px', fontSize: '12px' }}>
          [!] No dump files found. Click <strong>Autopwn</strong> or <strong>Dump</strong> above to create one — it will appear here automatically.
        </div>
      )}
    </>
  );
}