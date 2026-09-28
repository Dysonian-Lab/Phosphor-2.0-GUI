import { useAdvanced } from '../../hooks/useAdvanced';
import { useState, useEffect, useRef } from 'react';

export function AntennaStep({ advanced }: { advanced: ReturnType<typeof useAdvanced> }) {
  const [output, setOutput] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [vLf125, setVLf125] = useState<number | null>(null);
  const [vLf134, setVLf134] = useState<number | null>(null);
  const [vHf, setVHf] = useState<number | null>(null);
  const [peakV, setPeakV] = useState<number | null>(null);
  const [peakF, setPeakF] = useState<number | null>(null);
  const [lfQ, setLfQ] = useState<number | null>(null);
  const [judgement, setJudgement] = useState<string | null>(null);
  const [isSweeping, setIsSweeping] = useState(false);
  const sweepIntervalRef = useRef<number | null>(null);

  const getErrorMessage = (e: any): string => {
    if (typeof e === 'string') return e;
    if (e && typeof e === 'object') {
      const val = Object.values(e)[0];
      return typeof val === 'string' ? val : JSON.stringify(e);
    }
    return String(e);
  };

  const parseHwTuneOutput = (raw: string) => {
    const lines = raw.split('\n');
    let vLf125: number | null = null;
    let vLf134: number | null = null;
    let vHf: number | null = null;
    let peakV: number | null = null;
    let peakF: number | null = null;
    let lfQ: number | null = null;
    let judgement: string | null = null;

    for (const line of lines) {
      // LF 125 kHz: "125.00 kHz ........... 24.02 V"
      const lf125Match = line.match(/125\s*kHz\s*\.+\s*([\d.]+)\s*V/i);
      if (lf125Match) vLf125 = parseFloat(lf125Match[1]);

      // LF 134 kHz: "134.83 kHz ........... 16.05 V"
      const lf134Match = line.match(/134\s*kHz\s*\.+\s*([\d.]+)\s*V/i);
      if (lf134Match) vLf134 = parseFloat(lf134Match[1]);

      // HF 13.56 MHz: "13.56 MHz............. 60.79 V"
      const hfMatch = line.match(/13\.56\s+MHz\s*\.+\s*([\d.]+)\s*V/i);
      if (hfMatch) vHf = parseFloat(hfMatch[1]);

      // Peak: "120.00 kHz optimal.... 25.63 V"
      const peakMatch = line.match(/([\d.]+)\s*kHz\s+optimal\s*\.+\s*([\d.]+)\s*V/i);
      if (peakMatch) {
        peakF = parseFloat(peakMatch[1]);
        peakV = parseFloat(peakMatch[2]);
      }

      // Q factor: "Frequency bandwidth... 6.4"
      const qMatch = line.match(/Frequency bandwidth\s*\.+\s*([\d.]+)/i);
      if (qMatch) lfQ = parseFloat(qMatch[1]);

      // Judgement: "LF antenna............ ok" or "HF antenna ( ok )"
      if (line.match(/LF antenna\s*\.+\s*(ok|marginal|unusable)/i)) {
        const jm = line.match(/LF antenna\s*\.+\s*(ok|marginal|unusable)/i);
        if (jm) judgement = jm[1];
      } else if (line.match(/HF antenna\s*\(?\s*(ok|marginal|unusable)/i)) {
        const jm = line.match(/HF antenna\s*\(?\s*(ok|marginal|unusable)/i);
        if (jm && !judgement) judgement = jm[1];
      }
    }

    return { vLf125, vLf134, vHf, peakV, peakF, lfQ, judgement };
  };

  const runHwTune = async () => {
    try {
      const raw = await advanced.hwTune();
      const parsed = parseHwTuneOutput(raw);
      setVLf125(parsed.vLf125);
      setVLf134(parsed.vLf134);
      setVHf(parsed.vHf);
      setPeakV(parsed.peakV);
      setPeakF(parsed.peakF);
      setLfQ(parsed.lfQ);
      setJudgement(parsed.judgement);
      return raw;
    } catch (e: any) {
      throw e;
    }
  };

  const run = async (fn: () => Promise<string>, label: string) => {
    setLoading(true);
    setError(null);
    try {
      const result = await fn();
      setOutput(`=== ${label} ===\n${result}`);
    } catch (e: any) {
      setError(getErrorMessage(e));
    } finally {
      setLoading(false);
    }
  };

const sweep = async () => {
    if (isSweeping) {
      setIsSweeping(false);
      if (sweepIntervalRef.current) {
        clearInterval(sweepIntervalRef.current);
        sweepIntervalRef.current = null;
      }
      return;
    }
    setIsSweeping(true);
    // Run continuously every 12 seconds (hw tune takes ~6-10 seconds)
    const doSweep = async () => {
      try {
        await runHwTune();
      } catch (e) {
        setIsSweeping(false);
      }
    };
    sweepIntervalRef.current = setInterval(doSweep, 12000);
    await doSweep(); // run immediately
  };

  useEffect(() => {
    return () => {
      if (sweepIntervalRef.current) {
        clearInterval(sweepIntervalRef.current);
      }
    };
  }, []);

  const barStyle = (voltage: number | null, maxV: number, label: string, color: string) => {
    if (voltage === null) return <div style={{color: 'var(--red-bright)', fontFamily: 'var(--font-mono)'}}>N/A</div>;
    const pct = Math.min(100, (voltage / maxV) * 100);
    return (
      <div style={{ marginBottom: '4px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '2px' }}>
          <span style={{ width: '80px', fontFamily: 'var(--font-mono)', fontSize: '12px' }}>{label}</span>
          <span style={{ width: '60px', fontFamily: 'var(--font-mono)', fontSize: '12px', textAlign: 'right' }}>
            {voltage.toFixed(1)} V
          </span>
        </div>
        <div style={{ height: '18px', background: 'var(--bg-void)', borderRadius: '3px', border: '1px solid var(--green-dim)', overflow: 'hidden' }}>
          <div style={{ width: `${pct}%`, height: '100%', background: color, transition: 'width 0.2s ease-out' }} />
        </div>
      </div>
    );
  };

  return (
    <div style={{ maxWidth: '640px' }}>
      <h3>Antenna Measurement</h3>
      <p style={{ color: 'var(--text-secondary)', fontSize: '13px', marginBottom: '12px' }}>
        Measure antenna voltage, Q factor, and health. These are informative only — PM3 does not actively tune your antennas.
      </p>

      <div style={{ marginBottom: '12px', padding: '10px', background: 'var(--bg-tertiary)', borderRadius: '6px' }}>
        <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', marginBottom: '12px' }}>
          <button onClick={runHwTune} disabled={loading}
            style={{ background: 'var(--bg-void)', fontFamily: 'var(--font-mono)', fontSize: '12px', padding: '4px 12px', cursor: 'pointer', color: 'var(--green-bright)', border: '1px solid var(--green-bright)' }}>
            HW Tune
          </button>
          <button onClick={sweep} disabled={loading}
            style={{ background: 'var(--bg-void)', fontFamily: 'var(--font-mono)', fontSize: '12px', padding: '4px 12px', cursor: 'pointer', color: isSweeping ? 'var(--red-bright)' : 'var(--amber)', border: '1px solid ' + (isSweeping ? 'var(--red-bright)' : 'var(--amber)') }}>
            {isSweeping ? 'Stop Sweep' : 'Start Sweep'}
          </button>
        </div>

        <div style={{ marginBottom: '8px', fontSize: '11px', color: 'var(--amber)', fontFamily: 'var(--font-mono)' }}>
          [!] These bars show <b>antenna tuning voltages</b> (no card). They measure antenna resonance at 13.56 MHz / 125 kHz / 134 kHz.
          <br />For <b>card signal strength</b>, use the Mifare/ISO14443 tools with a card on the antenna.
        </div>

        {barStyle(vHf, 10, 'HF 13.56 MHz', 'var(--green-bright)')}
        {barStyle(vLf125, 10, 'LF 125 kHz', 'var(--green-mid)')}
        {barStyle(vLf134, 10, 'LF 134 kHz', 'var(--green-mid)')}
        {barStyle(peakV, 10, 'Peak V', 'var(--amber)')}

        <div style={{ marginTop: '8px', fontSize: '11px', color: 'var(--green-dim)', fontFamily: 'var(--font-mono)' }}>
          {peakF !== null && <span>Peak: {peakF} kHz | </span>}
          {lfQ !== null && <span>Q: {lfQ.toFixed(1)} | </span>}
          {judgement && <span style={{ color: judgement === 'ok' ? 'var(--green-bright)' : judgement === 'marginal' ? 'var(--amber)' : 'var(--red-bright)' }}>LF: {judgement.toUpperCase()}</span>}
        </div>
      </div>

      <div style={{ marginBottom: '12px', padding: '10px', background: 'var(--bg-tertiary)', borderRadius: '6px' }}>
        <div style={{ fontSize: '13px', marginBottom: '6px', color: 'var(--green-dim)' }}>3. HF Decay</div>
        <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap', marginBottom: '6px' }}>
          <button onClick={() => run(advanced.hwDecay, 'hw decay')} disabled={loading}
            style={{ background: 'var(--bg-void)', fontFamily: 'var(--font-mono)', fontSize: '12px', padding: '4px 12px', cursor: 'pointer', color: 'var(--green-mid)', border: '1px solid var(--green-mid)' }}>
            HW Decay
          </button>
        </div>
        <div style={{ color: 'var(--green-dim)', fontSize: '11px' }}>
          hw decay — measures how quickly the HF antenna voltage drops after the field turns off. Useful for detecting booster boards or damaged antennas.
        </div>
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