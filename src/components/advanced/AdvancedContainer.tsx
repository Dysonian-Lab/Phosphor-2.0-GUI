import { useAdvanced } from '../../hooks/useAdvanced';
import { Iso14bStep } from './Iso14bStep';
import { Iso15Step } from './Iso15Step';
import { FelicaStep } from './FelicaStep';
import { IclassSeStep } from './IclassSeStep';
import { LegicStep } from './LegicStep';
import { ScriptStep } from './ScriptStep';
import { FirmwareStep } from './FirmwareStep';
import { TuningStep } from './TuningStep';
import { AntennaStep } from './AntennaStep';
import React from 'react';

export function AdvancedContainer() {
  const advanced = useAdvanced();

  // Simple tab‑switch UI (you can replace with a more fancy Tabs component)
  const [active, setActive] = React.useState<'iso14b'|'iso15'|'felica'|'iclass'|'legic'|'script'|'firmware'|'tuning'|'antenna'>('iso14b');

  return (
    <div style={{ padding: '24px' }}>
      <h2>Advanced Tools</h2>
      <div style={{ display: 'flex', gap: '12px', marginBottom: '16px' }}>
        <button onClick={() => setActive('iso14b')}>ISO 14443‑B</button>
        <button onClick={() => setActive('iso15')}>ISO 15693</button>
        <button onClick={() => setActive('felica')}>Felica</button>
        <button onClick={() => setActive('iclass')}>iCLASS SE/SEOS</button>
        <button onClick={() => setActive('legic')}>LEGIC</button>
        <button onClick={() => setActive('script')}>Script / Lua</button>
        <button onClick={() => setActive('firmware')}>Firmware</button>
        <button onClick={() => setActive('tuning')}>Tuning</button>
        <button onClick={() => setActive('antenna')}>Antenna</button>
      </div>

      {/* Render the selected step */}
      {active === 'iso14b' && <Iso14bStep advanced={advanced} />}
      {active === 'iso15' && <Iso15Step advanced={advanced} />}
      {active === 'felica' && <FelicaStep advanced={advanced} />}
      {active === 'iclass' && <IclassSeStep advanced={advanced} />}
      {active === 'legic' && <LegicStep advanced={advanced} />}
      {active === 'script' && <ScriptStep advanced={advanced} />}
      {active === 'firmware' && <FirmwareStep advanced={advanced} />}
      {active === 'tuning' && <TuningStep advanced={advanced} />}
      {active === 'antenna' && <AntennaStep advanced={advanced} />}
    </div>
  );
}