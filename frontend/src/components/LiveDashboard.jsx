import React, { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

export default function LiveDashboard({ testId, onComplete, onError }) {
  const [metrics, setMetrics] = useState({
    completed: 0,
    total: 100,
    rps: 0,
    success: 0,
    fail: 0,
    avg: 0,
    p95: 0
  });
  
  const [history, setHistory] = useState([]);
  const [logs, setLogs] = useState([]);

  useEffect(() => {
    if (!testId) return;

    setLogs(prev => [...prev, `[INIT] Connecting to telemetry stream for test: ${testId}`]);
    const sse = new EventSource(`/api/stream/${testId}`);

    sse.onmessage = (e) => {
      try {
        const payload = JSON.parse(e.data);
        
        if (payload.type === 'progress') {
          setMetrics({
            completed: payload.completed,
            total: payload.total,
            rps: payload.rps,
            success: payload.successes,
            fail: payload.failures,
            avg: payload.avg_latency_ms,
            p95: payload.p95_latency_ms
          });

          setHistory(prev => {
            const newHistory = [...prev, {
              time: new Date().toLocaleTimeString(),
              latency: payload.avg_latency_ms,
              rps: payload.rps
            }];
            return newHistory.slice(-20); // keep last 20 frames
          });
          
          setLogs(prev => {
            const msgs = [...prev, `[INFO] Sent batch | RPS: ${payload.rps.toFixed(1)} | P95: ${payload.p95_latency_ms.toFixed(1)}ms`];
            return msgs.slice(-5);
          });
        }
        
        if (payload.type === 'done') {
          setLogs(prev => [...prev, `[DONE] Mission finished.`]);
          sse.close();
          setTimeout(() => onComplete(payload), 1000);
        }
        
        if (payload.type === 'error') {
          setLogs(prev => [...prev, `[ERROR] ${payload.message}`]);
          sse.close();
          onError(payload.message);
        }

      } catch (err) {
        console.error("Parse error", err);
      }
    };

    sse.onerror = () => {
      console.log('Telemetry stream disconnected');
      setLogs(prev => [...prev, `[WARN] Connection lost, retrying...`]);
      // The browser will automatically attempt to reconnect EventSource
    };

    return () => {
      sse.close();
    };
  }, [testId, onComplete, onError]);

  const progressPct = metrics.total > 0 ? (metrics.completed / metrics.total) * 100 : 0;

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 1.05 }}
      className="p-4 w-full max-w-5xl mx-auto space-y-6"
    >
      <div className="flex items-center justify-between border-b-[3px] border-[var(--border-color)] pb-4">
        <h2 className="text-3xl font-black uppercase tracking-tighter flex items-center gap-2">
          <span className="w-4 h-4 rounded-full bg-[var(--neo-danger)] animate-pulse inline-block"></span>
          Live Telemetry
        </h2>
        <div className="font-mono bg-[#1a1a1a] text-[var(--neo-accent)] px-3 py-1 font-bold">
          ID: {testId.split('-')[0]}
        </div>
      </div>

      <div className="bg-[var(--panel-bg)] border-[3px] border-[var(--border-color)] shadow-[6px_6px_0_var(--neo-shadow)] p-1 overflow-hidden">
        <div 
          className="h-6 bg-[var(--neo-primary)] transition-all ease-linear"
          style={{ width: `${progressPct}%` }}
        />
        <div className="text-center font-bold text-sm mt-1 uppercase border-t-[3px] border-[var(--border-color)] pt-1">
          Progress: {metrics.completed} / {metrics.total}
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
        <div className="neo-card flex flex-col items-center justify-center p-6 bg-[var(--neo-accent)] text-[#1a1a1a]">
          <span className="text-sm font-bold uppercase mb-2">Throughput</span>
          <span className="text-5xl font-black overflow-hidden">{metrics.rps.toFixed(1)}</span>
          <span className="text-xs font-bold uppercase mt-1">Req / Sec</span>
        </div>
        
        <div className="neo-card flex flex-col items-center justify-center p-6">
          <span className="text-sm font-bold uppercase mb-2">Average (p50)</span>
          <span className="text-4xl font-black font-mono">{metrics.avg.toFixed(1)}<span className="text-xl">ms</span></span>
        </div>

        <div className="neo-card flex flex-col items-center justify-center p-6">
          <span className="text-sm font-bold uppercase mb-2">Tail (p95)</span>
          <span className="text-4xl font-black font-mono text-[var(--neo-danger)]">{metrics.p95.toFixed(1)}<span className="text-xl">ms</span></span>
        </div>

        <div className="neo-card flex flex-col justify-center p-6 border-l-[8px] border-l-[var(--neo-secondary)]">
          <div className="flex justify-between font-bold w-full uppercase text-sm mb-2">
            <span>Success</span> <span className="text-[var(--neo-accent)]">{metrics.success}</span>
          </div>
          <div className="flex justify-between font-bold w-full uppercase text-sm">
            <span>Failed</span> <span className="text-[var(--neo-danger)]">{metrics.fail}</span>
          </div>
        </div>
      </div>

      <div className="neo-card p-4 h-[300px]">
        <h3 className="font-bold uppercase mb-4 text-sm border-b-[3px] border-[var(--border-color)] pb-2 inline-block">Latency Trend (ms)</h3>
        <ResponsiveContainer width="100%" height="85%">
          <LineChart data={history}>
            <CartesianGrid strokeDasharray="3 3" stroke="var(--border-color)" opacity={0.2} />
            <XAxis dataKey="time" hide />
            <YAxis stroke="var(--text-color)" tick={{fontFamily: 'JetBrains Mono', fontSize: 12}} width={40} />
            <Tooltip 
              contentStyle={{ 
                backgroundColor: 'var(--panel-bg)', 
                border: '3px solid var(--border-color)',
                boxShadow: '4px 4px 0 var(--neo-shadow)',
                borderRadius: '0',
                color: 'var(--text-color)',
                fontWeight: 'bold'
              }}
            />
            <Line type="monotone" dataKey="latency" stroke="var(--neo-primary)" strokeWidth={4} dot={false} isAnimationActive={false} />
          </LineChart>
        </ResponsiveContainer>
      </div>

      <div className="neo-card bg-[#1a1a1a] text-[var(--neo-accent)] p-4 font-mono text-sm shadow-none border-b-8 border-b-[var(--neo-primary)]">
        {logs.map((log, idx) => (
          <div key={idx} className="opacity-80 py-1 border-b border-[#333] last:border-0">{log}</div>
        ))}
      </div>
    </motion.div>
  );
}
