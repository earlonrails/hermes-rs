import React, { useState, useEffect } from 'react';

export default function ConfigView() {
  const [config, setConfig] = useState(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    fetch('/api/config')
      .then(r => r.json())
      .then(setConfig)
      .catch(console.error);
  }, []);

  const handleSave = async (e) => {
    e.preventDefault();
    setSaving(true);
    try {
      await fetch('/api/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config)
      });
      alert('Configuration saved successfully!');
    } catch (e) {
      alert('Error saving configuration: ' + e);
    }
    setSaving(false);
  };

  const handleAgentChange = (field, value) => {
    setConfig(prev => ({
      ...prev,
      agent: {
        ...(prev.agent || {}),
        [field]: value
      }
    }));
  };

  if (!config) return <div className="view-container"><p>Loading...</p></div>;

  return (
    <div className="view-container">
      <h1 className="page-title">Configuration</h1>
      
      <div className="card" style={{ marginBottom: '16px' }}>
        <h3 style={{ marginBottom: '12px', color: 'var(--accent-cyan)' }}>How to use this page</h3>
        <p style={{ color: 'var(--text-muted)', lineHeight: '1.5' }}>
          This page controls global Athena settings like the execution backend and agent behaviour loops. Model selection and API keys have moved to the Models tab. 
        </p>
      </div>

      <div className="card">
        <form onSubmit={handleSave} style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
          
          <div>
            <h3 style={{ marginBottom: '12px', color: 'var(--accent-cyan)' }}>Agent Settings</h3>
            
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginBottom: '16px' }}>
              <label style={{ fontSize: '0.9rem', color: 'var(--text-muted)' }}>Max Iterations</label>
              <input 
                type="number" 
                value={config.agent?.max_iterations || 20} 
                onChange={e => handleAgentChange('max_iterations', parseInt(e.target.value, 10))}
                style={{ padding: '10px', borderRadius: '8px', background: 'rgba(0,0,0,0.3)', color: '#fff', border: '1px solid var(--border-dim)' }}
              />
            </div>
            
            <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '16px' }}>
              <input 
                type="checkbox" 
                id="yolo_mode"
                checked={config.agent?.yolo_mode || false} 
                onChange={e => handleAgentChange('yolo_mode', e.target.checked)}
              />
              <label htmlFor="yolo_mode" style={{ fontSize: '0.9rem', color: 'var(--text-muted)' }}>YOLO Mode (Auto-approve dangerous commands)</label>
            </div>
          </div>

          <div>
            <h3 style={{ marginBottom: '12px', color: 'var(--accent-cyan)' }}>Terminal Backend</h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              <select 
                value={config.terminal_backend || 'local'} 
                onChange={e => setConfig(prev => ({ ...prev, terminal_backend: e.target.value }))}
                style={{ padding: '10px', borderRadius: '8px', background: 'rgba(0,0,0,0.3)', color: '#fff', border: '1px solid var(--border-dim)' }}
              >
                <option value="local">Local</option>
                <option value="docker">Docker</option>
                <option value="ssh">SSH</option>
                <option value="modal">Modal</option>
              </select>
            </div>
          </div>

          <div style={{ marginTop: '16px' }}>
            <button 
              type="submit" 
              disabled={saving}
              style={{ 
                padding: '10px 24px', 
                background: 'linear-gradient(135deg, var(--accent-indigo), #a855f7)',
                color: 'white',
                border: 'none',
                borderRadius: '8px',
                cursor: saving ? 'wait' : 'pointer',
                fontWeight: 'bold'
              }}
            >
              {saving ? 'Saving...' : 'Save Configuration'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
