import React, { useState, useEffect } from 'react';

export default function MCPView() {
  const [mcp, setMcp] = useState(null);
  const [saving, setSaving] = useState(false);

  const [newName, setNewName] = useState('');
  const [newCommand, setNewCommand] = useState('');
  const [newArgs, setNewArgs] = useState('');

  const fetchMcp = () => {
    fetch('/api/mcp')
      .then(r => r.json())
      .then(setMcp)
      .catch(console.error);
  };

  useEffect(() => {
    fetchMcp();
  }, []);

  const saveMcpList = async (newList) => {
    setSaving(true);
    try {
      await fetch('/api/mcp', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(newList)
      });
      fetchMcp();
    } catch (e) {
      alert('Error saving MCP: ' + e);
    }
    setSaving(false);
  };

  const handleToggle = (index) => {
    const clone = { ...mcp };
    clone.servers[index].enabled = !clone.servers[index].enabled;
    saveMcpList(clone);
  };

  const handleRemove = (index) => {
    const clone = { ...mcp };
    clone.servers.splice(index, 1);
    saveMcpList(clone);
  };

  const handleAdd = (e) => {
    e.preventDefault();
    if (!newName || !newCommand) return;
    
    const argsList = newArgs.split(' ').filter(a => a.trim() !== '');
    const newServer = {
      name: newName,
      command: newCommand,
      args: argsList,
      enabled: true
    };
    
    const clone = { ...mcp };
    if (!clone.servers) clone.servers = [];
    clone.servers.push(newServer);
    
    saveMcpList(clone);
    setNewName('');
    setNewCommand('');
    setNewArgs('');
  };

  if (!mcp) return <div className="view-container"><p>Loading...</p></div>;

  return (
    <div className="view-container">
      <h1 className="page-title">MCP Servers</h1>
      
      <div className="card" style={{ marginBottom: '16px' }}>
        <h3 style={{ marginBottom: '12px', color: 'var(--accent-cyan)' }}>How to use this page</h3>
        <p style={{ color: 'var(--text-muted)', lineHeight: '1.5' }}>
          The Model Context Protocol (MCP) enables Athena to query external local services, tools, and databases. Add a new MCP server by specifying its executable command. You can easily toggle servers on or off depending on your needs.
        </p>
      </div>

      <div className="card" style={{ marginBottom: '24px' }}>
        <h3 style={{ marginBottom: '16px', color: '#fff' }}>Add New Server</h3>
        <form onSubmit={handleAdd} style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr auto', gap: '12px', alignItems: 'end' }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
            <label style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Name</label>
            <input 
              type="text" required placeholder="e.g. fetch-docs"
              value={newName} onChange={e => setNewName(e.target.value)}
              style={{ padding: '8px 12px', borderRadius: '6px', background: 'rgba(0,0,0,0.3)', color: '#fff', border: '1px solid var(--border-dim)' }}
            />
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
            <label style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Command</label>
            <input 
              type="text" required placeholder="e.g. npx"
              value={newCommand} onChange={e => setNewCommand(e.target.value)}
              style={{ padding: '8px 12px', borderRadius: '6px', background: 'rgba(0,0,0,0.3)', color: '#fff', border: '1px solid var(--border-dim)' }}
            />
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
            <label style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Args (space separated)</label>
            <input 
              type="text" placeholder="-y @modelcontextprotocol/server-postgres"
              value={newArgs} onChange={e => setNewArgs(e.target.value)}
              style={{ padding: '8px 12px', borderRadius: '6px', background: 'rgba(0,0,0,0.3)', color: '#fff', border: '1px solid var(--border-dim)' }}
            />
          </div>
          <button type="submit" disabled={saving} style={{ padding: '9px 16px', background: 'var(--accent-indigo)', color: 'white', border: 'none', borderRadius: '6px', cursor: 'pointer', fontWeight: 'bold' }}>
            Add
          </button>
        </form>
      </div>

      <div className="card">
        <h3 style={{ marginBottom: '16px', color: '#fff' }}>Configured Servers</h3>
        {(!mcp.servers || mcp.servers.length === 0) ? (
          <p style={{ color: 'var(--text-muted)' }}>No servers configured.</p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            {mcp.servers.map((s, i) => (
              <div key={i} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '16px', background: 'rgba(0,0,0,0.2)', borderRadius: '8px', border: '1px solid var(--border-dim)' }}>
                <div>
                  <h4 style={{ color: s.enabled ? '#fff' : 'var(--text-muted)', marginBottom: '4px' }}>{s.name}</h4>
                  <p style={{ fontSize: '0.85rem', color: 'var(--text-dim)', fontFamily: 'monospace' }}>
                    $ {s.command} {s.args.join(' ')}
                  </p>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                  <button 
                    onClick={() => handleToggle(i)}
                    disabled={saving}
                    style={{ padding: '6px 12px', background: s.enabled ? 'rgba(16, 185, 129, 0.2)' : 'rgba(255,255,255,0.05)', color: s.enabled ? '#10b981' : 'var(--text-muted)', border: `1px solid ${s.enabled ? 'rgba(16, 185, 129, 0.3)' : 'var(--border-dim)'}`, borderRadius: '6px', cursor: 'pointer' }}
                  >
                    {s.enabled ? 'Enabled' : 'Disabled'}
                  </button>
                  <button 
                    onClick={() => handleRemove(i)}
                    disabled={saving}
                    style={{ padding: '6px 12px', background: 'rgba(239, 68, 68, 0.1)', color: '#ef4444', border: '1px solid rgba(239, 68, 68, 0.2)', borderRadius: '6px', cursor: 'pointer' }}
                  >
                    Remove
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
