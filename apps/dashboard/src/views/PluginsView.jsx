import React, { useState, useEffect } from 'react';

export default function PluginsView() {
  const [plugins, setPlugins] = useState([]);
  const [saving, setSaving] = useState(false);
  const [newName, setNewName] = useState('');

  const fetchPlugins = () => {
    fetch('/api/plugins')
      .then(r => r.json())
      .then(setPlugins)
      .catch(console.error);
  };

  useEffect(() => {
    fetchPlugins();
  }, []);

  const handleAdd = async (e) => {
    e.preventDefault();
    if (!newName.trim()) return;
    setSaving(true);
    try {
      await fetch('/api/plugins', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: newName })
      });
      fetchPlugins();
      setNewName('');
    } catch (err) {
      alert('Error adding plugin: ' + err);
    }
    setSaving(false);
  };

  const handleRemove = async (name) => {
    if (!confirm(`Are you sure you want to remove the plugin ${name}?`)) return;
    setSaving(true);
    try {
      await fetch(`/api/plugins/${name}`, { method: 'DELETE' });
      fetchPlugins();
    } catch (err) {
      alert('Error removing plugin: ' + err);
    }
    setSaving(false);
  };

  return (
    <div className="view-container">
      <h1 className="page-title">Plugins</h1>
      
      <div className="card" style={{ marginBottom: '16px' }}>
        <h3 style={{ marginBottom: '12px', color: 'var(--accent-cyan)' }}>How to use this page</h3>
        <p style={{ color: 'var(--text-muted)', lineHeight: '1.5' }}>
          Plugins extend Athena's runtime natively through sandboxed WebAssembly execution. Use this page to register a new `.wasm` plugin skeleton, which you can compile and deploy to `~/.athena/plugins/`.
        </p>
      </div>

      <div className="card" style={{ marginBottom: '24px' }}>
        <h3 style={{ marginBottom: '16px', color: '#fff' }}>Register New Plugin Skeleton</h3>
        <form onSubmit={handleAdd} style={{ display: 'flex', gap: '12px', alignItems: 'end' }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', flex: 1 }}>
            <label style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Plugin Identifier</label>
            <input 
              type="text" required placeholder="e.g. custom-parser, code-analyzer"
              value={newName} onChange={e => setNewName(e.target.value)}
              style={{ padding: '8px 12px', borderRadius: '6px', background: 'rgba(0,0,0,0.3)', color: '#fff', border: '1px solid var(--border-dim)' }}
            />
          </div>
          <button type="submit" disabled={saving} style={{ padding: '9px 16px', background: 'var(--accent-indigo)', color: 'white', border: 'none', borderRadius: '6px', cursor: 'pointer', fontWeight: 'bold' }}>
            Register Skeleton
          </button>
        </form>
      </div>

      <div className="card">
        <h3 style={{ marginBottom: '16px', color: '#fff' }}>Installed Plugins</h3>
        {plugins.length > 0 ? (
          <ul style={{ listStyleType: 'none', padding: 0, display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {plugins.map((p, i) => (
              <li key={i} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '12px 16px', background: 'rgba(0,0,0,0.2)', borderRadius: '8px', border: '1px solid var(--border-dim)' }}>
                <div>
                  <strong style={{ color: '#fff' }}>{p.name}</strong> <br/>
                  <small style={{ color: 'var(--text-muted)', fontFamily: 'monospace' }}>{p.path}</small>
                </div>
                <button 
                  onClick={() => handleRemove(p.name)}
                  disabled={saving}
                  style={{ padding: '6px 12px', background: 'rgba(239, 68, 68, 0.1)', color: '#ef4444', border: '1px solid rgba(239, 68, 68, 0.2)', borderRadius: '6px', cursor: 'pointer' }}
                >
                  Remove
                </button>
              </li>
            ))}
          </ul>
        ) : (
          <p style={{ color: 'var(--text-muted)' }}>No plugins found in ~/.athena/plugins.</p>
        )}
      </div>
    </div>
  );
}
