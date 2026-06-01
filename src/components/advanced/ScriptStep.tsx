import { useAdvanced } from '../../hooks/useAdvanced';
import { useState, useEffect } from 'react';
import CodeMirror from '@uiw/react-codemirror';
import { StreamLanguage } from '@codemirror/language';
import { lua } from '@codemirror/legacy-modes/mode/lua';
import { highlightActiveLine } from '@codemirror/view';

export function ScriptStep({ advanced }: { advanced: ReturnType<typeof useAdvanced> }) {
  const [scripts, setScripts] = useState<string[]>([]);
  const [selectedScript, setSelectedScript] = useState<string>(''); // empty means new script
  const [scriptContent, setScriptContent] = useState<string>('');
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

  // Load the list of scripts on mount and when needed
  useEffect(() => {
    loadScripts();
  }, []);

  const loadScripts = async () => {
    setLoading(true);
    setError(null);
    try {
      const scriptList = await advanced.listScripts();
      setScripts(scriptList);
      // If we have a selected script that is no longer in the list, reset to empty (new script)
      if (selectedScript && !scripts.includes(selectedScript)) {
        setSelectedScript('');
      }
    } catch (e: any) {
      setError(getErrorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  const loadScript = async (filename: string) => {
    setLoading(true);
    setError(null);
    try {
      const content = await advanced.readScript(filename);
      setScriptContent(content);
      setSelectedScript(filename);
    } catch (e: any) {
      setError(getErrorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  const handleLoadClick = async () => {
    if (selectedScript) {
      await loadScript(selectedScript);
    }
  };

  const saveScript = async () => {
    setLoading(true);
    setError(null);
    try {
      if (selectedScript) {
        // Save to the selected script
        await advanced.writeScript(selectedScript, scriptContent);
      } else {
        // We don't have a filename yet; we should prompt the user for a new filename.
        // For simplicity, we'll use a prompt. In a real app, we might use a modal.
        const filename = window.prompt('Enter a filename for the new script (without extension):', '');
        if (filename === null) {
          // User cancelled
          setLoading(false);
          return;
        }
        const trimmed = filename.trim();
        if (trimmed === '') {
          alert('Filename cannot be empty');
          setLoading(false);
          return;
        }
        // We'll add .lua extension if not present? Let's keep as is, but we can enforce .lua.
        // For now, we'll just use the given filename.
        await advanced.writeScript(trimmed, scriptContent);
        setSelectedScript(trimmed);
        // Refresh the script list to include the new script
        await loadScripts();
      }
    } catch (e: any) {
      setError(getErrorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  const runScript = async () => {
    setLoading(true);
    setError(null);
    setOutput(null);
    try {
      const result = await advanced.runScript(scriptContent);
      setOutput(result.output);
    } catch (e: any) {
      setError(getErrorMessage(e));
    } finally {
      setLoading(false);
    }
  };

  const handleScriptChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    if (value === '--new-script--') {
      setSelectedScript('');
      setScriptContent('');
    } else {
      setSelectedScript(value);
      // Load the selected script
      loadScript(value);
    }
  };

  return (
    <div style={{ maxWidth: '600px' }}>
      <h3>Lua Script Runner</h3>
      <div style={{ marginBottom: '12px', display: 'flex', gap: '8px', alignItems: 'center' }}>
        <label htmlFor='script-select'>Script: </label>
        <select
          id='script-select'
          value={selectedScript === '' ? '--new-script--' : selectedScript}
          onChange={handleScriptChange}
          disabled={loading}
          style={{ minWidth: '200px' }}
        >
          <option value='--new-script--'>-- New Script --</option>
          {scripts.map((script) => (
            <option key={script} value={script}>
              {script}
            </option>
          ))}
        </select>
        <button onClick={handleLoadClick} disabled={loading || selectedScript === ''}>
          Load
        </button>
        <button onClick={saveScript} disabled={loading}>
          Save
        </button>
        <button onClick={() => {
          setSelectedScript('');
          setScriptContent('');
        }} disabled={loading}>
          New
        </button>
      </div>

      {/* CodeMirror Editor with Lua syntax highlighting and line numbers */}
      <div
        style={{
          width: '100%',
          height: '400px', // Fixed height for the editor
        }}
      >
        <CodeMirror
          value={scriptContent}
          height='400px'
          basicSetup={{ lineNumbers: true }}
          extensions={[StreamLanguage.define(lua), highlightActiveLine()]}
          onChange={(val) => setScriptContent(val)}
        />
      </div>

      <br />
      <button onClick={runScript} disabled={loading}>
        {loading ? 'Running...' : 'Run Script'}
      </button>

      {error && (
        <div style={{ color: 'var(--red-bright)', marginTop: '12px' }}>
          Error: {error}
        </div>
      )}

      {output && (
        <div style={{ marginTop: '16px', fontFamily: 'var(--font-mono)', whiteSpace: 'pre-wrap' }}>
          <strong>Output:</strong><br />
          {output}
        </div>
      )}
    </div>
  );
}
