import React, { useState, useEffect } from 'react';

export default function ModelView() {
  const [config, setConfig] = useState(null);
  const [saving, setSaving] = useState(false);
  const providers = ['openai', 'anthropic', 'gemini', 'mistral', 'openrouter', 'deepseek', 'groq', 'xai'];

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
      alert('Models configuration saved successfully!');
    } catch (e) {
      alert('Error saving configuration: ' + e);
    }
    setSaving(false);
  };

  const handleChange = (section, field, value) => {
    setConfig(prev => ({
      ...prev,
      [section]: {
        ...prev[section],
        [field]: value
      }
    }));
  };

  const handleProviderKeyChange = (providerSlug, key) => {
    setConfig(prev => {
      const providersMap = prev.providers || {};
      const providerInfo = providersMap[providerSlug] || { name: providerSlug };
      
      return {
        ...prev,
        providers: {
          ...providersMap,
          [providerSlug]: {
            ...providerInfo,
            api_key: key
          }
        }
      };
    });
  };

  if (!config) return <div className="view-container"><p>Loading...</p></div>;

  return (
    <div className="view-container">
      <h1 className="page-title">Models</h1>
      
      <div className="card" style={{ marginBottom: '16px' }}>
        <h3 style={{ marginBottom: '12px', color: 'var(--accent-cyan)' }}>How to use this page</h3>
        <p style={{ color: 'var(--text-muted)', lineHeight: '1.5' }}>
          Select your primary LLM provider and default model below. In order for Athena to communicate with the provider, you must provide the respective API key. API keys are stored securely in your local `~/.athena/config.yaml` file.
        </p>
      </div>

      <div className="card">
        <form onSubmit={handleSave} style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
          
          <div>
            <h3 style={{ marginBottom: '16px', color: '#fff' }}>Active Model</h3>
            
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginBottom: '16px' }}>
              <label style={{ fontSize: '0.9rem', color: 'var(--text-muted)' }}>Default Provider</label>
              <select 
                value={config.model?.provider || ''} 
                onChange={e => handleChange('model', 'provider', e.target.value)}
                style={{ padding: '10px', borderRadius: '8px', background: 'rgba(0,0,0,0.3)', color: '#fff', border: '1px solid var(--border-dim)' }}
              >
                {providers.map(p => <option key={p} value={p}>{p.charAt(0).toUpperCase() + p.slice(1)}</option>)}
              </select>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              <label style={{ fontSize: '0.9rem', color: 'var(--text-muted)' }}>Default Model (e.g. gpt-4o, claude-3-opus-20240229)</label>
              <input 
                type="text" 
                value={config.model?.default || ''} 
                onChange={e => handleChange('model', 'default', e.target.value)}
                style={{ padding: '10px', borderRadius: '8px', background: 'rgba(0,0,0,0.3)', color: '#fff', border: '1px solid var(--border-dim)' }}
              />
            </div>
          </div>

          <hr style={{ borderColor: 'var(--border-dim)' }} />

          <div>
            <h3 style={{ marginBottom: '16px', color: '#fff' }}>API Keys</h3>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: '16px' }}>
              {providers.map(provider => {
                const pInfo = config.providers?.[provider] || {};
                return (
                  <div key={provider} style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                    <label style={{ fontSize: '0.9rem', color: 'var(--text-muted)' }}>{provider.charAt(0).toUpperCase() + provider.slice(1)} API Key</label>
                    <input 
                      type="password" 
                      placeholder={`Enter ${provider} key`}
                      value={pInfo.api_key || ''} 
                      onChange={e => handleProviderKeyChange(provider, e.target.value)}
                      style={{ padding: '10px', borderRadius: '8px', background: 'rgba(0,0,0,0.3)', color: '#fff', border: '1px solid var(--border-dim)' }}
                    />
                  </div>
                );
              })}
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
              {saving ? 'Saving...' : 'Save Models Configuration'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
