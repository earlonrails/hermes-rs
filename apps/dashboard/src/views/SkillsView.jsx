import React, { useState, useEffect } from 'react';

export default function SkillsView() {
  const [skills, setSkills] = useState([]);
  const [saving, setSaving] = useState(false);
  const [newName, setNewName] = useState('');

  const fetchSkills = () => {
    fetch('/api/skills')
      .then(r => r.json())
      .then(setSkills)
      .catch(console.error);
  };

  useEffect(() => {
    fetchSkills();
  }, []);

  const handleAdd = async (e) => {
    e.preventDefault();
    if (!newName.trim()) return;
    setSaving(true);
    try {
      await fetch('/api/skills', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: newName })
      });
      fetchSkills();
      setNewName('');
    } catch (err) {
      alert('Error adding skill: ' + err);
    }
    setSaving(false);
  };

  const handleRemove = async (name) => {
    if (!confirm(`Are you sure you want to remove the skill ${name}?`)) return;
    setSaving(true);
    try {
      await fetch(`/api/skills/${name}`, { method: 'DELETE' });
      fetchSkills();
    } catch (err) {
      alert('Error removing skill: ' + err);
    }
    setSaving(false);
  };

  return (
    <div className="view-container">
      <h1 className="page-title">Skills</h1>
      
      <div className="card" style={{ marginBottom: '16px' }}>
        <h3 style={{ marginBottom: '12px', color: 'var(--accent-cyan)' }}>How to use this page</h3>
        <p style={{ color: 'var(--text-muted)', lineHeight: '1.5' }}>
          Semantic Skills give your agent new dynamic capabilities without hardcoding them into the main binary. You can register new `.rs` skill templates here, which you can then customize in your `~/.athena/skills/` directory.
        </p>
      </div>

      <div className="card" style={{ marginBottom: '24px' }}>
        <h3 style={{ marginBottom: '16px', color: '#fff' }}>Register New Skill Template</h3>
        <form onSubmit={handleAdd} style={{ display: 'flex', gap: '12px', alignItems: 'end' }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', flex: 1 }}>
            <label style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Skill Identifier</label>
            <input 
              type="text" required placeholder="e.g. notify-slack, fetch-docs"
              value={newName} onChange={e => setNewName(e.target.value)}
              style={{ padding: '8px 12px', borderRadius: '6px', background: 'rgba(0,0,0,0.3)', color: '#fff', border: '1px solid var(--border-dim)' }}
            />
          </div>
          <button type="submit" disabled={saving} style={{ padding: '9px 16px', background: 'var(--accent-indigo)', color: 'white', border: 'none', borderRadius: '6px', cursor: 'pointer', fontWeight: 'bold' }}>
            Register Template
          </button>
        </form>
      </div>

      <div className="card">
        <h3 style={{ marginBottom: '16px', color: '#fff' }}>Installed Skills</h3>
        {skills.length > 0 ? (
          <ul style={{ listStyleType: 'none', padding: 0, display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {skills.map((s, i) => (
              <li key={i} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '12px 16px', background: 'rgba(0,0,0,0.2)', borderRadius: '8px', border: '1px solid var(--border-dim)' }}>
                <div>
                  <strong style={{ color: '#fff' }}>{s.name}</strong> <br/>
                  <small style={{ color: 'var(--text-muted)', fontFamily: 'monospace' }}>{s.path}</small>
                </div>
                <button 
                  onClick={() => handleRemove(s.name)}
                  disabled={saving}
                  style={{ padding: '6px 12px', background: 'rgba(239, 68, 68, 0.1)', color: '#ef4444', border: '1px solid rgba(239, 68, 68, 0.2)', borderRadius: '6px', cursor: 'pointer' }}
                >
                  Remove
                </button>
              </li>
            ))}
          </ul>
        ) : (
          <p style={{ color: 'var(--text-muted)' }}>No skills found in ~/.athena/skills.</p>
        )}
      </div>
    </div>
  );
}
