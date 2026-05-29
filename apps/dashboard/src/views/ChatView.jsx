import React, { useState, useEffect, useRef } from 'react';

export default function ChatView() {
  const [messages, setMessages] = useState([
    { role: 'agent', content: 'Hello! I am Athena. How can I help you today?' }
  ]);
  const [input, setInput] = useState('');
  const [ws, setWs] = useState(null);
  const [config, setConfig] = useState(null);
  const endRef = useRef(null);

  const fetchConfig = () => {
    fetch('/api/config')
      .then(r => r.json())
      .then(setConfig)
      .catch(console.error);
  };

  useEffect(() => {
    fetchConfig();
    
    // Attempt to connect to websocket
    const socket = new WebSocket(`ws://${window.location.host}/api/chat`);
    
    socket.onopen = () => {
      console.log('Connected to chat server');
    };

    socket.onmessage = (event) => {
      setMessages(prev => [...prev, { role: 'agent', content: event.data }]);
    };

    socket.onclose = () => {
      console.log('Disconnected from chat server');
    };

    setWs(socket);

    return () => socket.close();
  }, []);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const sendMessage = (e) => {
    e.preventDefault();
    if (!input.trim() || !ws) return;

    const text = input.trim();
    setMessages(prev => [...prev, { role: 'user', content: text }]);
    ws.send(text);
    setInput('');
  };

  const changeModel = async (newProvider) => {
    if (!config) return;
    const clone = { ...config };
    if (!clone.model) clone.model = {};
    clone.model.provider = newProvider;
    
    // Set a decent default model for the chosen provider
    if (newProvider === 'openai') clone.model.default = 'gpt-4o';
    else if (newProvider === 'anthropic') clone.model.default = 'claude-3-5-sonnet-20240620';
    else if (newProvider === 'mistral') clone.model.default = 'mistral-large-latest';
    else clone.model.default = '';

    try {
      await fetch('/api/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(clone)
      });
      setConfig(clone);
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div className="view-container" style={{ height: '100vh', display: 'flex', flexDirection: 'column' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
        <h1 className="page-title" style={{ marginBottom: 0 }}>Agent Chat</h1>
        {config && (
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
            <span style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>Active Model:</span>
            <select 
              value={config.model?.provider || ''} 
              onChange={e => changeModel(e.target.value)}
              style={{ padding: '6px 12px', borderRadius: '6px', background: 'rgba(0,0,0,0.3)', color: 'var(--accent-cyan)', border: '1px solid var(--border-dim)', fontWeight: 'bold' }}
            >
              <option value="openai">OpenAI ({config.model?.provider === 'openai' ? config.model?.default : 'gpt-4o'})</option>
              <option value="anthropic">Anthropic ({config.model?.provider === 'anthropic' ? config.model?.default : 'claude-3-5-sonnet'})</option>
              <option value="gemini">Gemini</option>
              <option value="mistral">Mistral ({config.model?.provider === 'mistral' ? config.model?.default : 'mistral-large'})</option>
              <option value="openrouter">OpenRouter</option>
              <option value="deepseek">DeepSeek</option>
              <option value="groq">Groq</option>
              <option value="xai">xAI</option>
            </select>
          </div>
        )}
      </div>

      <div className="chat-window" style={{ flex: 1 }}>
        <div className="chat-messages">
          {messages.map((msg, i) => (
            <div key={i} className={`chat-bubble ${msg.role}`}>
              {msg.content}
            </div>
          ))}
          <div ref={endRef} />
        </div>
        <form className="chat-input-area" onSubmit={sendMessage}>
          <input 
            type="text" 
            placeholder="Type your message..." 
            value={input}
            onChange={(e) => setInput(e.target.value)}
          />
          <button type="submit">Send</button>
        </form>
      </div>
    </div>
  );
}
