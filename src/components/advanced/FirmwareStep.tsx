import { useAdvanced } from '../../hooks/useAdvanced';
import { useState } from 'react';

export function FirmwareStep({ advanced }: { advanced: ReturnType<typeof useAdvanced> }) {
  const [variant, setVariant] = useState<'rdv4' | 'rdv4-bt' | 'generic'>('rdv4');
  const [progress, setProgress] = useState<number>(0);
  const [message, setMessage] = useState<string>('Ready to flash');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const flashFirmware = async () => {
    setLoading(true);
    setError(null);
    setProgress(0);
    setMessage('Starting flash...');
    try {
      // In a real implementation, we would get the port from wizard context
      // For now, we'll use a placeholder since we don't have direct access to wizard context here
      const port = "COM3"; // This should come from wizard context in a real implementation
      await advanced.flashFirmware(port, variant);
      setMessage('Flash complete!');
      setProgress(100);
    } catch (e: any) {
      setError(e?.message ?? String(e));
      setMessage('Flash failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <h3>Firmware Flashing</h3>
      <div>
        <label>
          Variant:
          <select
            value={variant}
            onChange={(e) => setVariant(e.target.value as 'rdv4' | 'rdv4-bt' | 'generic')}
          >
            <option value="rdv4">RDV4</option>
            <option value="rdv4-bt">RDV4-BT</option>
            <option value="generic">Generic</option>
          </select>
        </label>
      </div>
      <div style={{ marginTop: '16px' }}>
        <button onClick={flashFirmware} disabled={loading}>
          {loading ? 'Flashing…' : 'Flash Firmware'}
        </button>
      </div>

      {error && (
        <div style={{ color: 'var(--red-bright)', marginTop: '12px' }}>
          Error: {error}
        </div>
      )}

      {!loading && progress > 0 && message !== 'Ready to flash' && (
        <div style={{ marginTop: '12px' }}>
          <div>Progress: {progress}%</div>
          <div>{message}</div>
        </div>
      )}
    </div>
  );
}