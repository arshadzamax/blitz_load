import React, { useState } from 'react';
import { motion } from 'framer-motion';

export default function ConfigPanel({ onLaunch, isLoading }) {
  const [config, setConfig] = useState({
    url: 'https://httpbin.org/post',
    method: 'post',
    requests: 100,
    concurrency: 10,
    scenario: 'user_registration',
    custom_body: '',
    timeout_ms: 30000,
    sla_ms: ''
  });

  const handleSubmit = (e) => {
    e.preventDefault();
    const payload = { ...config };
    if (payload.sla_ms === '') payload.sla_ms = null;
    onLaunch(payload);
  };

  const handleChange = (e) => {
    const { name, value, type } = e.target;
    setConfig(prev => ({
      ...prev,
      [name]: (type === 'number' || type === 'range') ? Number(value) : value
    }));
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      className="max-w-2xl mx-auto w-full p-4"
    >
      <div className="neo-card p-6 md:p-8 relative">
        <div className="absolute -top-[15px] -left-[15px] bg-[var(--neo-accent)] text-[#1a1a1a] font-black px-4 py-1 border-[3px] border-[#1a1a1a] transform -rotate-3 uppercase">
          Configure Mission
        </div>
        
        <form onSubmit={handleSubmit} className="space-y-6 mt-4">
          <div className="flex flex-col md:flex-row gap-4">
            <div className="flex-1 space-y-2">
              <label className="font-bold uppercase text-sm">Target URL</label>
              <input
                type="url"
                name="url"
                required
                value={config.url}
                onChange={handleChange}
                className="neo-input w-full font-mono text-sm"
                placeholder="https://api.example.com"
              />
            </div>
            
            <div className="w-full md:w-32 space-y-2">
              <label className="font-bold uppercase text-sm">Method</label>
              <select
                name="method"
                value={config.method}
                onChange={handleChange}
                className="neo-input w-full font-bold cursor-pointer transition-none appearance-none"
              >
                <option value="post">POST</option>
                <option value="get">GET</option>
              </select>
            </div>
          </div>

          <div className="flex flex-col md:flex-row gap-4">
            <div className="flex-1 space-y-2">
              <label className="font-bold uppercase text-sm flex justify-between">
                <span>Requests</span>
                <span className="text-[var(--neo-secondary)] font-mono">{config.requests}</span>
              </label>
              <input
                type="range"
                name="requests"
                min="10"
                max="10000"
                step="10"
                value={config.requests}
                onChange={handleChange}
                className="w-full h-3 bg-[var(--bg-color)] rounded-none appearance-none border-2 border-[var(--border-color)] outline-none"
              />
            </div>

            <div className="flex-1 space-y-2">
              <label className="font-bold uppercase text-sm flex justify-between">
                <span>Concurrency</span>
                <span className="text-[var(--neo-accent)] font-mono">{config.concurrency}</span>
              </label>
              <input
                type="range"
                name="concurrency"
                min="1"
                max="1000"
                value={config.concurrency}
                onChange={handleChange}
                className="w-full h-3 bg-[var(--bg-color)] rounded-none appearance-none border-2 border-[var(--border-color)] outline-none"
              />
            </div>
          </div>

          <div className="space-y-2">
            <label className="font-bold uppercase text-sm">Scenario Format</label>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
              {[
                { id: 'user_registration', label: 'UUID User Auth' },
                { id: 'simple_get', label: 'Simple No-Body' },
                { id: 'custom_json', label: 'Custom JSON' }
              ].map(opt => (
                <label 
                  key={opt.id} 
                  className={`border-[3px] border-[var(--border-color)] p-3 text-center font-bold text-sm cursor-pointer transition-colors ${config.scenario === opt.id ? 'bg-[var(--neo-primary)] text-[#1a1a1a] shadow-[4px_4px_0_var(--neo-shadow)] -translate-y-1 -translate-x-1' : 'bg-[var(--panel-bg)] hover:bg-[var(--bg-color)]'}`}
                >
                  <input
                    type="radio"
                    name="scenario"
                    value={opt.id}
                    checked={config.scenario === opt.id}
                    onChange={handleChange}
                    className="hidden"
                  />
                  {opt.label}
                </label>
              ))}
            </div>
          </div>

          {config.scenario === 'custom_json' && (
            <motion.div 
              initial={{ height: 0, opacity: 0 }}
              animate={{ height: 'auto', opacity: 1 }}
              className="space-y-2"
            >
              <label className="font-bold uppercase text-sm">Custom JSON Body</label>
              <textarea
                name="custom_body"
                value={config.custom_body}
                onChange={handleChange}
                className="neo-input w-full font-mono text-sm h-32 resize-y"
                placeholder='{"key": "value"}'
              />
            </motion.div>
          )}

          <div className="flex flex-col md:flex-row gap-4">
            <div className="flex-1 space-y-2">
              <label className="font-bold uppercase text-sm">Timeout (ms)</label>
              <input
                type="number"
                name="timeout_ms"
                required
                min="100"
                value={config.timeout_ms}
                onChange={handleChange}
                className="neo-input w-full font-mono text-sm"
              />
            </div>
            
            <div className="flex-1 space-y-2">
              <label className="font-bold uppercase text-sm">SLA Limit (ms)</label>
              <input
                type="number"
                name="sla_ms"
                min="10"
                value={config.sla_ms}
                onChange={handleChange}
                className="neo-input w-full font-mono text-sm"
                placeholder="Optional"
              />
            </div>
          </div>

          <div className="pt-4">
            <button
              type="submit"
              disabled={isLoading}
              className={`neo-button w-full py-4 text-xl w-full neo-button-primary ${isLoading ? 'opacity-50 cursor-not-allowed' : ''}`}
            >
              {isLoading ? 'INITIATING BARRAGE...' : 'LAUNCH BARRAGE ⚡'}
            </button>
          </div>
        </form>
      </div>
    </motion.div>
  );
}
