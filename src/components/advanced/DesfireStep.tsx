import { useAdvanced } from '../../hooks/useAdvanced';
import { useState } from 'react';
import { DumpFilePicker } from './DumpFilePicker';

/** Shared authentication options, matching the `hf mfdes` help output. */
const ALGOS = ['DES', '2TDEA', '3TDEA', 'AES'];
const KDFS = ['none', 'AN10922', 'gallagher'];
const CMODES = ['plain', 'mac', 'encrypt'];
const CCSETS = ['native', 'niso', 'iso'];
const SCHANNS = ['d40', 'ev1', 'ev2', 'lrp'];
const FILE_TYPES = ['auto', 'data', 'value', 'record', 'mac'];
const VALUE_OPS = ['get', 'credit', 'limcredit', 'debit', 'clear'];

/**
 * Subcommands handled by `desfire_auth`. Each entry lists the shared
 * authentication flags the command accepts; the backend also drops any
 * subcommand-specific extra the chosen command does not understand.
 */
const AUTH_COMMANDS: { sub: string; label: string; note?: string }[] = [
  { sub: 'auth', label: 'auth', note: 'authenticate only' },
  { sub: 'getuid', label: 'getuid' },
  { sub: 'freemem', label: 'freemem', note: 'free memory' },
  { sub: 'lsapp', label: 'lsapp', note: 'list applications' },
  { sub: 'getaids', label: 'getaids' },
  { sub: 'getappnames', label: 'getappnames' },
  { sub: 'selectapp', label: 'selectapp' },
  { sub: 'getfileids', label: 'getfileids' },
  { sub: 'getfileisoids', label: 'getfileisoids' },
  { sub: 'getfilesettings', label: 'getfilesettings' },
  { sub: 'getkeysettings', label: 'getkeysettings' },
  { sub: 'getkeyversions', label: 'getkeyversions' },
  { sub: 'lsfiles', label: 'lsfiles' },
  { sub: 'mad', label: 'mad', note: 'MIFARE Application Directory' },
  { sub: 'setconfig', label: 'setconfig' },
  { sub: 'formatpicc', label: 'formatpicc', note: 'DESTRUCTIVE' },
  { sub: 'getdelegateappinfo', label: 'getdelegateappinfo' },
  { sub: 'createdelegateapp', label: 'createdelegateapp' },
  { sub: 'selectisofid', label: 'selectisofid' },
  { sub: 'clearrecfile', label: 'clearrecfile' },
  { sub: 'chfilesettings', label: 'chfilesettings' },
  { sub: 'chkeysettings', label: 'chkeysettings' },
  { sub: 'deletefile', label: 'deletefile', note: 'DESTRUCTIVE' },
  { sub: 'deleteapp', label: 'deleteapp', note: 'DESTRUCTIVE' },
  { sub: 'brutedamslot', label: 'brutedamslot' },
  { sub: 'createmacfile', label: 'createmacfile' },
  { sub: 'createvaluefile', label: 'createvaluefile' },
  { sub: 'createrecordfile', label: 'createrecordfile' },
  { sub: 'createmfcmapping', label: 'createmfcmapping' },
];

/** Subcommands that genuinely take no options at all. */
const BARE_COMMANDS: { sub: string; label: string; note?: string }[] = [
  { sub: 'info', label: 'info', note: 'card information' },
  { sub: 'getversion', label: 'getversion' },
  { sub: 'eview', label: 'eview', note: 'emulator memory image' },
  { sub: 'sim', label: 'sim', note: 'simulate loaded image' },
  { sub: 'test', label: 'test', note: 'self test' },
];

const btn: React.CSSProperties = {
  background: 'var(--bg-void)',
  fontFamily: 'var(--font-mono)',
  fontSize: '12px',
  padding: '4px 12px',
  cursor: 'pointer',
  color: 'var(--green-bright)',
  border: '1px solid var(--green-bright)',
};

const inputStyle: React.CSSProperties = {
  background: 'var(--bg-void)',
  border: '1px solid var(--green-dim)',
  color: 'var(--green-bright)',
  fontFamily: 'var(--font-mono)',
  fontSize: '12px',
  padding: '3px 6px',
  borderRadius: '3px',
  minWidth: '90px',
};

const labelStyle: React.CSSProperties = {
  fontSize: '11px',
  color: 'var(--green-dim)',
  display: 'block',
  marginBottom: '2px',
};

export function DesfireStep({ advanced }: { advanced: ReturnType<typeof useAdvanced> }) {
  const [output, setOutput] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);

  // Shared authentication state.
  const [sub, setSub] = useState('getuid');
  const [algo, setAlgo] = useState('');
  const [key, setKey] = useState('');
  const [kdf, setKdf] = useState('');
  const [kdfi, setKdfi] = useState('');
  const [cmode, setCmode] = useState('');
  const [ccset, setCcset] = useState('');
  const [schann, setSchann] = useState('');
  const [aid, setAid] = useState('');
  const [keyno, setKeyno] = useState('');
  const [noAuth, setNoAuth] = useState(false);
  const [apdu, setApdu] = useState(false);
  const [verbose, setVerbose] = useState(false);

  // Generic extras.
  const [isoid, setIsoid] = useState('');
  const [dfname, setDfname] = useState('');
  const [fid, setFid] = useState('');
  const [isofid, setIsofid] = useState('');
  const [dataHex, setDataHex] = useState('');

  // Per-command inputs.
  const [isoidV, setIsoidV] = useState('');
  const [fidV, setFidV] = useState('');
  const [ftypeV, setFtypeV] = useState('');
  const [offsetV, setOffsetV] = useState('');
  const [dataV, setDataV] = useState('');
  const [chkAid, setChkAid] = useState('');
  const [chkKey, setChkKey] = useState('');
  const [chkKdf, setChkKdf] = useState('');
  const [chkPattern, setChkPattern] = useState('');
  const [chkJson, setChkJson] = useState('');
  const [dumpKeys, setDumpKeys] = useState('');
  const [dumpLength, setDumpLength] = useState('');
  const [fileName, setFileName] = useState('');
  const [keepOnSave, setKeepOnSave] = useState(false);
  const [etestAction, setEtestAction] = useState('state');
  const [etestApdu, setEtestApdu] = useState('');
  const [etestJson, setEtestJson] = useState(false);
  const [pcKey, setPcKey] = useState('');
  const [pcRetry, setPcRetry] = useState('');
  const [valueOp, setValueOp] = useState('get');
  const [trBuffer, setTrBuffer] = useState(false);
  const [trFrame, setTrFrame] = useState(false);
  const [trCrc, setTrCrc] = useState(false);
  const [trRelative, setTrRelative] = useState(false);
  const [trMicro, setTrMicro] = useState(false);
  const [trPcap, setTrPcap] = useState(false);

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
      if (label === 'dump' || label === 'esave') setRefreshKey((k) => k + 1);
    } catch (e: any) {
      setError(getErrorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  const authBase = () => ({
    algo: algo || null,
    key: key || null,
    kdf: kdf || null,
    kdfi: kdfi || null,
    cmode: cmode || null,
    ccset: ccset || null,
    schann: schann || null,
    aid: aid || null,
    keyno: keyno || null,
    noAuth,
    apdu,
    verbose,
  });

  const runAuth = () =>
    run(
      () =>
        advanced.desfireAuth({
          sub,
          ...authBase(),
          isoid: isoid || null,
          dfname: dfname || null,
          fid: fid || null,
          isofid: isofid || null,
          data: dataHex || null,
          noAuth,
        }),
      `mfdes ${sub}`,
    );

  return (
    <div style={{ maxWidth: '900px' }}>
      <h3>DESFire (EV1 / EV2 / EV3)</h3>
      <p style={{ color: 'var(--text-secondary)', fontSize: '13px', marginBottom: '12px' }}>
        MIFARE DESFire commands for Proxmark3 v4.23346. Options marked{' '}
        <code>hf mfdes</code> are sent to the client exactly as shown; the client
        reports its own errors. Leave a field blank to omit that option.
      </p>

      {/* Quick actions */}
      <Section title="Quick actions">
        <Row>
          {BARE_COMMANDS.map((c) => (
            <button
              key={c.sub}
              style={btn}
              disabled={loading}
              title={c.note}
              onClick={() => run(() => advanced.desfireBare(c.sub), `mfdes ${c.sub}`)}
            >
              {c.label}
            </button>
          ))}
        </Row>
      </Section>

      {/* Authentication + command dispatch */}
      <Section title="Authenticated command">
        <p style={{ fontSize: '11px', color: 'var(--text-secondary)', margin: '0 0 6px' }}>
          The shared authentication flags are sent to whichever command is
          selected. Extra fields are only emitted for commands that accept them.
        </p>
        <Grid>
          <Field label="command">
            <select style={inputStyle} value={sub} onChange={(e) => setSub(e.target.value)}>
              {AUTH_COMMANDS.map((c) => (
                <option key={c.sub} value={c.sub}>
                  {c.label}
                  {c.note ? ` — ${c.note}` : ''}
                </option>
              ))}
            </select>
          </Field>
          <Field label="-t algo">
            <select style={inputStyle} value={algo} onChange={(e) => setAlgo(e.target.value)}>
              <option value="">(default)</option>
              {ALGOS.map((a) => (
                <option key={a} value={a}>{a}</option>
              ))}
            </select>
          </Field>
          <Field label="-k key">
            <input style={inputStyle} value={key} placeholder="16/32/48 hex"
              onChange={(e) => setKey(e.target.value)} />
          </Field>
          <Field label="--kdf">
            <select style={inputStyle} value={kdf} onChange={(e) => setKdf(e.target.value)}>
              <option value="">(default)</option>
              {KDFS.map((k) => (
                <option key={k} value={k}>{k}</option>
              ))}
            </select>
          </Field>
          <Field label="-i kdfi">
            <input style={inputStyle} value={kdfi} onChange={(e) => setKdfi(e.target.value)} />
          </Field>
          <Field label="-m cmode">
            <select style={inputStyle} value={cmode} onChange={(e) => setCmode(e.target.value)}>
              <option value="">(default)</option>
              {CMODES.map((c) => (
                <option key={c} value={c}>{c}</option>
              ))}
            </select>
          </Field>
          <Field label="-c ccset">
            <select style={inputStyle} value={ccset} onChange={(e) => setCcset(e.target.value)}>
              <option value="">(default)</option>
              {CCSETS.map((c) => (
                <option key={c} value={c}>{c}</option>
              ))}
            </select>
          </Field>
          <Field label="--schann">
            <select style={inputStyle} value={schann} onChange={(e) => setSchann(e.target.value)}>
              <option value="">(default)</option>
              {SCHANNS.map((c) => (
                <option key={c} value={c}>{c}</option>
              ))}
            </select>
          </Field>
          <Field label="--aid">
            <input style={inputStyle} value={aid} placeholder="3 bytes"
              onChange={(e) => setAid(e.target.value)} />
          </Field>
          <Field label="-n keyno">
            <input style={inputStyle} value={keyno} onChange={(e) => setKeyno(e.target.value)} />
          </Field>
          <Field label="--isoid">
            <input style={inputStyle} value={isoid} placeholder="2 bytes"
              onChange={(e) => setIsoid(e.target.value)} />
          </Field>
          <Field label="--dfname">
            <input style={inputStyle} value={dfname} placeholder="hex"
              onChange={(e) => setDfname(e.target.value)} />
          </Field>
          <Field label="--fid">
            <input style={inputStyle} value={fid} placeholder="1 byte"
              onChange={(e) => setFid(e.target.value)} />
          </Field>
          <Field label="--isofid">
            <input style={inputStyle} value={isofid} onChange={(e) => setIsofid(e.target.value)} />
          </Field>
          <Field label="-d data">
            <input style={inputStyle} value={dataHex} onChange={(e) => setDataHex(e.target.value)} />
          </Field>
        </Grid>
        <Checks
          items={[
            ['--no-auth', noAuth, setNoAuth],
            ['-a apdu', apdu, setApdu],
            ['-v verbose', verbose, setVerbose],
          ]}
        />
        <RunButton onClick={runAuth} disabled={loading} label={`Run: hf mfdes ${sub}`} />
      </Section>

      {/* detect / chk / dump */}
      <Section title="Detect, key check and dump">
        <Row>
          <button style={btn} disabled={loading}
            onClick={() => run(
              () => advanced.desfireDetect({
                ...authBase(),
                isoid: isoid || null,
                dfname: dfname || null,
                noAuth,
              }),
              'mfdes detect')}>
            detect
          </button>
          <button style={btn} disabled={loading}
            onClick={() => run(
              () => advanced.desfireDump({
                ...authBase(),
                isoid: isoid || null,
                dfname: dfname || null,
                length: dumpLength || null,
                keys: dumpKeys || null,
                noAuth,
              }),
              'dump')}>
            dump
          </button>
        </Row>
        <Grid>
          <Field label="dump -l length">
            <input style={inputStyle} value={dumpLength} placeholder="3 bytes"
              onChange={(e) => setDumpLength(e.target.value)} />
          </Field>
          <Field label="dump --keys">
            <input style={inputStyle} value={dumpKeys} placeholder="keys.json"
              onChange={(e) => setDumpKeys(e.target.value)} />
          </Field>
        </Grid>

        <SubHead>chk — check keys (dictionary brute force)</SubHead>
        <Grid>
          <Field label="--aid">
            <input style={inputStyle} value={chkAid} onChange={(e) => setChkAid(e.target.value)} />
          </Field>
          <Field label="-k key">
            <input style={inputStyle} value={chkKey} onChange={(e) => setChkKey(e.target.value)} />
          </Field>
          <Field label="--kdf 0/1/2">
            <select style={inputStyle} value={chkKdf} onChange={(e) => setChkKdf(e.target.value)}>
              <option value="">(default)</option>
              <option value="0">0 none</option>
              <option value="1">1 AN10922</option>
              <option value="2">2 gallagher</option>
            </select>
          </Field>
          <Field label="pattern">
            <select style={inputStyle} value={chkPattern} onChange={(e) => setChkPattern(e.target.value)}>
              <option value="">(none)</option>
              <option value="pattern1b">--pattern1b</option>
              <option value="pattern2b">--pattern2b</option>
            </select>
          </Field>
          <Field label="-j json out">
            <input style={inputStyle} value={chkJson} onChange={(e) => setChkJson(e.target.value)} />
          </Field>
        </Grid>
        <RunButton
          disabled={loading}
          label="Run: hf mfdes chk"
          onClick={() => run(
            () => {
              const args: any = {
                aid: chkAid || null,
                key: chkKey || null,
                kdf: chkKdf || null,
                kdfi: kdfi || null,
                schann: schann || null,
                pattern1b: chkPattern === 'pattern1b',
                pattern2b: chkPattern === 'pattern2b',
                json: chkJson || null,
                apdu,
                verbose,
              };
              if (chkPattern === 'pattern2b') args.startp2b = null;
              return advanced.desfireChk(args);
            },
            'mfdes chk')}
        />
      </Section>

      {/* read / write / value */}
      <Section title="Read, write and value files">
        <Grid>
          <Field label="--fid">
            <input style={inputStyle} value={fidV} onChange={(e) => setFidV(e.target.value)} />
          </Field>
          <Field label="--isoid">
            <input style={inputStyle} value={isoidV} onChange={(e) => setIsoidV(e.target.value)} />
          </Field>
          <Field label="--type">
            <select style={inputStyle} value={ftypeV} onChange={(e) => setFtypeV(e.target.value)}>
              <option value="">(auto)</option>
              {FILE_TYPES.map((t) => (
                <option key={t} value={t}>{t}</option>
              ))}
            </select>
          </Field>
          <Field label="-o offset">
            <input style={inputStyle} value={offsetV} placeholder="3 bytes"
              onChange={(e) => setOffsetV(e.target.value)} />
          </Field>
          <Field label="-d data">
            <input style={inputStyle} value={dataV} onChange={(e) => setDataV(e.target.value)} />
          </Field>
        </Grid>
        <Row>
          <button style={btn} disabled={loading}
            onClick={() => run(
              () => advanced.desfireRead({
                ...authBase(),
                fid: fidV || null,
                isoid: isoidV || null,
                ftype: ftypeV || null,
                offset: offsetV || null,
                noAuth,
              }),
              'mfdes read')}>
            read
          </button>
          <button style={btn} disabled={loading}
            onClick={() => run(
              () => advanced.desfireWrite({
                ...authBase(),
                fid: fidV || null,
                isoid: isoidV || null,
                ftype: ftypeV || null,
                offset: offsetV || null,
                data: dataV || null,
                noAuth,
              }),
              'mfdes write')}>
            write
          </button>
          <button style={btn} disabled={loading}
            onClick={() => run(
              () => advanced.desfireValue({
                ...authBase(),
                fid: fidV || null,
                isoid: isoidV || null,
                operation: valueOp || null,
                data: dataV || null,
                noAuth,
              }),
              'mfdes value')}>
            value
          </button>
        </Row>
        <Grid>
          <Field label="value -o operation">
            <select style={inputStyle} value={valueOp} onChange={(e) => setValueOp(e.target.value)}>
              {VALUE_OPS.map((o) => (
                <option key={o} value={o}>{o}</option>
              ))}
            </select>
          </Field>
        </Grid>
      </Section>

      {/* emulator */}
      <Section title="Emulator image (self-contained DESFire EV1 simulation)">
        <Grid>
          <Field label="dump file">
            <DumpFilePicker
              advanced={advanced}
              value={fileName}
              onChange={setFileName}
              disabled={loading}
              placeholder="e.g. hf-mfdes-01020304050607-dump.json"
              triggerRefresh={refreshKey}
            />
          </Field>
        </Grid>
        <Row>
          <button style={btn} disabled={loading || !fileName}
            onClick={() => run(() => advanced.desfireFile('eload', fileName), 'eload')}>
            eload
          </button>
          <button style={btn} disabled={loading}
            onClick={() => run(() => advanced.desfireFile('esave', fileName, keepOnSave), 'esave')}>
            esave
          </button>
          <button style={btn} disabled={loading || !fileName}
            onClick={() => run(() => advanced.desfireFile('view', fileName), 'view')}>
            view
          </button>
          <button style={btn} disabled={loading}
            onClick={() => run(() => advanced.desfireBare('eview'), 'eview')}>
            eview
          </button>
          <button style={btn} disabled={loading}
            onClick={() => run(() => advanced.desfireBare('sim'), 'sim')}>
            sim
          </button>
        </Row>
        <Checks items={[['esave --keep', keepOnSave, setKeepOnSave]]} />

        <SubHead>etest — drive the simulation from the host (no RF)</SubHead>
        <Grid>
          <Field label="action">
            <select style={inputStyle} value={etestAction} onChange={(e) => setEtestAction(e.target.value)}>
              <option value="state">--state</option>
              <option value="begin">--begin</option>
              <option value="scan">--scan</option>
              <option value="apdu">--apdu</option>
              <option value="random">--random</option>
              <option value="fieldoff">--fieldoff</option>
              <option value="end">--end</option>
            </select>
          </Field>
          <Field label="--apdu hex">
            <input style={inputStyle} value={etestApdu} onChange={(e) => setEtestApdu(e.target.value)} />
          </Field>
        </Grid>
        <RunButton
          disabled={loading}
          label="Run: hf mfdes etest"
          onClick={() => run(
            () => advanced.desfireEtest(etestAction, { apdu: etestApdu, json: etestJson }),
            'mfdes etest')}
        />
        <Checks items={[['-j json', etestJson, setEtestJson]]} />
      </Section>

      {/* card configuration */}
      <Section title="Card configuration">
        <Grid>
          <Field label="pc -k key">
            <input style={inputStyle} value={pcKey} onChange={(e) => setPcKey(e.target.value)} />
          </Field>
          <Field label="pc -r retry">
            <input style={inputStyle} value={pcRetry} onChange={(e) => setPcRetry(e.target.value)} />
          </Field>
        </Grid>
        <Row>
          <button style={btn} disabled={loading || !pcKey}
            onClick={() => run(
              () => advanced.desfirePc({ key: pcKey, retry: pcRetry || null, apdu, verbose }),
              'mfdes pc')}>
            pc
          </button>
          <button style={btn} disabled={loading}
            onClick={() => run(() => advanced.desfireAuth({ sub: 'setconfig', ...authBase(), noAuth }), 'mfdes setconfig')}>
            setconfig
          </button>
          <button style={btn} disabled={loading}
            onClick={() => run(() => advanced.desfireAuth({ sub: 'formatpicc', ...authBase(), noAuth }), 'mfdes formatpicc')}>
            formatpicc
          </button>
        </Row>
      </Section>

      {/* trace */}
      <Section title="Trace (hf mfdes list)">
        <Checks
          items={[
            ['-1 buffer', trBuffer, setTrBuffer],
            ['--frame', trFrame, setTrFrame],
            ['-c crc', trCrc, setTrCrc],
            ['-r relative', trRelative, setTrRelative],
            ['-u microseconds', trMicro, setTrMicro],
            ['-x pcap', trPcap, setTrPcap],
          ]}
        />
        <RunButton
          disabled={loading}
          label="Run: hf mfdes list"
          onClick={() => run(
            () => advanced.desfireList({
              buffer: trBuffer,
              frame: trFrame,
              crc: trCrc,
              relative: trRelative,
              microseconds: trMicro,
              pcap: trPcap,
            }),
            'mfdes list')}
        />
      </Section>

      {error && (<div style={{ color: 'var(--red-bright)', marginTop: '12px' }}>Error: {error}</div>)}
      {output && (
        <div style={{ marginTop: '16px', fontFamily: 'var(--font-mono)', whiteSpace: 'pre-wrap', background: 'var(--bg-tertiary)', padding: '12px', borderRadius: '6px' }}>
          {output}
        </div>
      )}
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div style={{ marginBottom: '14px', padding: '10px', background: 'var(--bg-tertiary)', borderRadius: '6px' }}>
      <div style={{ fontSize: '13px', marginBottom: '8px', color: 'var(--green-dim)' }}>{title}</div>
      {children}
    </div>
  );
}

function SubHead({ children }: { children: React.ReactNode }) {
  return (
    <div style={{ fontSize: '12px', margin: '10px 0 6px', color: 'var(--green-mid)' }}>
      {children}
    </div>
  );
}

function Row({ children }: { children: React.ReactNode }) {
  return <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px', marginBottom: '8px' }}>{children}</div>;
}

function Grid({ children }: { children: React.ReactNode }) {
  return (
    <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px', marginBottom: '8px' }}>
      {children}
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div>
      <span style={labelStyle}>{label}</span>
      {children}
    </div>
  );
}

function RunButton({ onClick, disabled, label }: { onClick: () => void; disabled: boolean; label: string }) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        ...btn,
        fontWeight: 600,
        padding: '4px 16px',
        borderWidth: 2,
        cursor: disabled ? 'default' : 'pointer',
      }}
    >
      {label}
    </button>
  );
}

function Checks({
  items,
}: {
  items: [string, boolean, (v: boolean) => void][];
}) {
  return (
    <div style={{ display: 'flex', flexWrap: 'wrap', gap: '10px', marginBottom: '8px' }}>
      {items.map(([label, value, onChange]) => (
        <label key={label} style={{ fontSize: '11px', color: 'var(--green-dim)', display: 'flex', gap: '4px', alignItems: 'center' }}>
          <input type="checkbox" checked={value} onChange={(e) => onChange(e.target.checked)} />
          {label}
        </label>
      ))}
    </div>
  );
}
