import React, { useState, useEffect } from 'react';

export default function ToolsView() {
  const [config, setConfig] = useState(null);
  const [saving, setSaving] = useState(false);
  const coreTools = ["filesystem_read", "filesystem_write", "web_search", "command_execution", "browser_automation"];

  const fetchConfig = () => {
    fetch('/api/config')
      .then(r => r.json())
      .then(setConfig)
      .catch(console.error);
  };

  useEffect(() => {
    fetchConfig();
  }, []);

  const handleToggle = async (tool) => {
    if (!config) return;
    setSaving(true);
    
    const clone = JSON.parse(JSON.stringify(config));
    if (!clone.tools) clone.tools = { disabled: [] };
    if (!clone.tools.disabled) clone.tools.disabled = [];
    
    if (clone.tools.disabled.includes(tool)) {
      clone.tools.disabled = clone.tools.disabled.filter(t => t !== tool);
    } else {
      clone.tools.disabled.push(tool);
    }

    try {
      await fetch('/api/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(clone)
      });
      fetchConfig();
    } catch (err) {
      alert('Error updating tools: ' + err);
    }
    setSaving(false);
  };

  if (!config) return <div className="view-container"><p>Loading...</p></div>;

  return (
    <div className="view-container">
      <h1 className="page-title">Tools</h1>
      
      <div className="card" style={{ marginBottom: '16px' }}>
        <h3 style={{ marginBottom: '12px', color: 'var(--accent-cyan)' }}>How to use this page</h3>
        <p style={{ color: 'var(--text-muted)', lineHeight: '1.5' }}>
          This page manages the core native tools available to Athena. You can toggle capabilities like file system access, execution environments, and web browsing to securely sandbox your agent workflows.
        </p>
      </div>

      <div className="card">
        <h3 style={{ marginBottom: '16px', color: '#fff' }}>Core Capabilities</h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          {coreTools.map(tool => {
            const isDisabled = config.tools?.disabled?.includes(tool);
            return (
              <div key={tool} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '16px', background: 'rgba(0,0,0,0.2)', borderRadius: '8px', border: '1px solid var(--border-dim)' }}>
                <div>
                  <h4 style={{ color: !isDisabled ? '#fff' : 'var(--text-muted)', marginBottom: '4px' }}>{tool}</h4>
                  <p style={{ fontSize: '0.85rem', color: 'var(--text-dim)' }}>
                    Core Athena tool capability.
                  </p>
                </div>
                <button 
                  onClick={() => handleToggle(tool)}
                  disabled={saving}
                  style={{ 
                    padding: '6px 12px', 
                    background: !isDisabled ? 'rgba(16, 185, 129, 0.2)' : 'rgba(255,255,255,0.05)', 
                    color: !isDisabled ? '#10b981' : 'var(--text-muted)', 
                    border: `1px solid ${!isDisabled ? 'rgba(16, 185, 129, 0.3)' : 'var(--border-dim)'}`, 
                    borderRadius: '6px', 
                    cursor: 'pointer' 
                  }}
                >
                  {!isDisabled ? 'Enabled' : 'Disabled'}
                </button>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
