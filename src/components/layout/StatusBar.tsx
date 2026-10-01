import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useSfx } from '../../hooks/useSfx';

export type SystemStatus = 'ready' | 'busy' | 'error';

interface StatusBarProps {
  status: SystemStatus;
  message?: string;
  musicEnabled: boolean;
  onMusicToggle: () => void;
}

function formatTime(d: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

function formatDate(d: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

function getStatusDisplay(status: SystemStatus, message?: string) {
  switch (status) {
    case 'ready':
      return {
        prefix: '[>>]',
        text: message || 'READY',
        color: 'var(--green-bright)',
      };
    case 'busy':
      return {
        prefix: '[!!]',
        text: message || 'BUSY',
        color: 'var(--amber)',
      };
    case 'error':
      return {
        prefix: '[XX]',
        text: message || 'ERROR',
        color: 'var(--red-bright)',
      };
  }
}

export function StatusBar({ status, message, musicEnabled, onMusicToggle }: StatusBarProps) {
  const [now, setNow] = useState(new Date());
  const [logPath, setLogPath] = useState<string | null>(null);
  const [logCopied, setLogCopied] = useState(false);

  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  // Make the diagnostic log trivially findable. Testers were previously asked
  // for "the log" without ever being told where it was, and it was written to a
  // path that did not even exist on their machine.
  const revealLog = async () => {
    try {
      const p = await invoke<string | null>('get_session_log_path');
      setLogPath(p ?? 'Log unavailable');
    } catch {
      setLogPath('Log unavailable');
    }
  };

  const copyLog = async () => {
    if (!logPath) return;
    try {
      await navigator.clipboard.writeText(logPath);
      setLogCopied(true);
      setTimeout(() => setLogCopied(false), 2000);
    } catch {
      setLogCopied(false);
    }
  };

  const sfx = useSfx();
  const display = getStatusDisplay(status, message);

  return (
    <div
      style={{
        height: '24px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 20px',
        background: 'var(--bg-panel)',
        borderTop: '1px solid var(--green-dim)',
        fontFamily: 'var(--font-mono)',
        fontSize: '11px',
        position: 'relative',
        zIndex: 10,
      }}
    >
      <div style={{
        color: display.color,
        overflow: 'hidden',
        textOverflow: 'ellipsis',
        whiteSpace: 'nowrap',
        maxWidth: '50%',
      }}>
        {display.prefix} {display.text}
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '20px' }}>
        <span
          onClick={() => { sfx.click(); onMusicToggle(); }}
          onMouseEnter={sfx.hover}
          style={{
            cursor: 'pointer',
            color: musicEnabled ? 'var(--green-bright)' : 'var(--green-dim)',
            userSelect: 'none',
            transition: 'color 0.15s',
          }}
          title={musicEnabled ? 'Music ON' : 'Music OFF'}
        >
          {musicEnabled ? '[♪ ON]' : '[♪ OFF]'}
        </span>
        <span
          onClick={logPath ? copyLog : revealLog}
          onMouseEnter={sfx.hover}
          style={{
            cursor: 'pointer',
            color: logCopied ? 'var(--green-bright)' : 'var(--green-dim)',
            userSelect: 'none',
            transition: 'color 0.15s',
          }}
          title={logPath ? 'Click to copy the log file path' : 'Click to find the diagnostic log file'}
        >
          {logCopied ? '[LOG PATH COPIED]' : '[LOG]'}
        </span>
        <span style={{ color: 'var(--green-dim)', letterSpacing: '0.5px' }}>
          {formatDate(now)}
          <span style={{ display: 'inline-block', width: '12px' }} />
          {formatTime(now)}
        </span>
      </div>
      {logPath && (
        <div
          style={{
            position: 'absolute',
            bottom: '26px',
            right: '12px',
            maxWidth: '70vw',
            padding: '6px 10px',
            background: 'var(--bg-void)',
            border: '1px solid var(--green-dim)',
            color: 'var(--green-bright)',
            fontFamily: 'var(--font-mono)',
            fontSize: '11px',
            wordBreak: 'break-all',
            zIndex: 20,
          }}
          onClick={copyLog}
          title="Click to copy"
        >
          Send this file when reporting a problem:{logPath}
        </div>
      )}
    </div>
  );
}
