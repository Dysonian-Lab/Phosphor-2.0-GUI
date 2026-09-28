import { Iso14bStep } from './Iso14bStep';
import { Iso15Step } from './Iso15Step';
import { FelicaStep } from './FelicaStep';
import { IclassSeStep } from './IclassSeStep';
import { LegicStep } from './LegicStep';
import { MfViewStep } from './MfViewStep';
import { CalypsoStep } from './CalypsoStep';
import { ThinfilmStep } from './ThinfilmStep';
import { MadStep } from './MadStep';
import { NfcStep } from './NfcStep';
import { MfuStep } from './MfuStep';
import { EmrtdStep } from './EmrtdStep';
import { SmartcardStep } from './SmartcardStep';
import { AntifuzzStep } from './AntifuzzStep';
import { T55xxStep } from './T55xxStep';
import { TraceStep } from './TraceStep';
import { ScriptStep } from './ScriptStep';
import { FirmwareStep } from './FirmwareStep';
import { TuningStep } from './TuningStep';
import { AntennaStep } from './AntennaStep';
import React from 'react';
import { useAdvanced } from '../../hooks/useAdvanced';

export function AdvancedContainer() {
  const advanced = useAdvanced();

  const [active, setActive] = React.useState<'iso14b'|'iso15'|'felica'|'iclass'|'legic'|'mfview'|'calypso'|'thinfilm'|'mad'|'nfc'|'mfu'|'emrtd'|'smartcard'|'antifuzz'|'t55xx'|'trace'|'script'|'firmware'|'tuning'|'antenna'>('iso14b');

  return (
    <div style={{ padding: '24px' }}>
      <h2>Advanced Tools</h2>
      <div style={{ display: 'flex', gap: '6px', marginBottom: '16px', flexWrap: 'wrap' }}>
        <button onClick={() => setActive('iso14b')}>14B</button>
        <button onClick={() => setActive('iso15')}>15693</button>
        <button onClick={() => setActive('felica')}>Felica</button>
        <button onClick={() => setActive('iclass')}>iCLASS</button>
        <button onClick={() => setActive('legic')}>LEGIC</button>
        <button onClick={() => setActive('mfview')}>Mifare View</button>
        <button onClick={() => setActive('calypso')}>Calypso</button>
        <button onClick={() => setActive('thinfilm')}>ThinFilm</button>
        <button onClick={() => setActive('mad')}>MAD</button>
        <button onClick={() => setActive('nfc')}>NFC</button>
        <button onClick={() => setActive('mfu')}>MFU</button>
        <button onClick={() => setActive('emrtd')}>eMRTD</button>
        <button onClick={() => setActive('smartcard')}>Smart</button>
        <button onClick={() => setActive('antifuzz')}>Antifuzz</button>
        <button onClick={() => setActive('t55xx')}>T55xx</button>
        <button onClick={() => setActive('trace')}>Trace</button>
        <button onClick={() => setActive('script')}>Lua Scripts</button>
        <button onClick={() => setActive('firmware')}>Firmware</button>
        <button onClick={() => setActive('tuning')}>Tuning</button>
        <button onClick={() => setActive('antenna')}>Antenna</button>
      </div>

      {active === 'iso14b' && <Iso14bStep advanced={advanced} />}
      {active === 'iso15' && <Iso15Step advanced={advanced} />}
      {active === 'felica' && <FelicaStep advanced={advanced} />}
      {active === 'iclass' && <IclassSeStep advanced={advanced} />}
      {active === 'legic' && <LegicStep advanced={advanced} />}
      {active === 'mfview' && <MfViewStep advanced={advanced} />}
      {active === 'calypso' && <CalypsoStep advanced={advanced} />}
      {active === 'thinfilm' && <ThinfilmStep advanced={advanced} />}
      {active === 'mad' && <MadStep advanced={advanced} />}
      {active === 'nfc' && <NfcStep advanced={advanced} />}
      {active === 'mfu' && <MfuStep advanced={advanced} />}
      {active === 'emrtd' && <EmrtdStep advanced={advanced} />}
      {active === 'smartcard' && <SmartcardStep advanced={advanced} />}
      {active === 'antifuzz' && <AntifuzzStep advanced={advanced} />}
      {active === 't55xx' && <T55xxStep advanced={advanced} />}
      {active === 'trace' && <TraceStep advanced={advanced} />}
      {active === 'script' && <ScriptStep advanced={advanced} />}
      {active === 'firmware' && <FirmwareStep advanced={advanced} />}
      {active === 'tuning' && <TuningStep advanced={advanced} />}
      {active === 'antenna' && <AntennaStep advanced={advanced} />}
    </div>
  );
}