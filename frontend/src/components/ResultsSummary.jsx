import React from 'react';
import { motion } from 'framer-motion';

export default function ResultsSummary({ results, onReset }) {
  if (!results) return null;

  const {
    completed, total, successes, failures,
    total_time_ms, rps, avg_latency_ms,
    p50_latency_ms, p95_latency_ms, p99_latency_ms, spread_us
  } = results;

  const successRate = total > 0 ? ((successes / total) * 100).toFixed(1) : 0;

  return (
    <motion.div
      initial={{ opacity: 0, y: 30 }}
      animate={{ opacity: 1, y: 0 }}
      className="p-4 w-full max-w-4xl mx-auto space-y-8"
    >
      <div className="text-center mb-8">
        <h2 className="text-5xl font-black uppercase mb-4 tracking-tighter flex justify-center items-center gap-4">
          <span className="text-[var(--neo-accent)]">✓</span> Mission Complete
        </h2>
        <p className="font-mono opacity-80 bg-[var(--panel-bg)] px-4 py-2 border-[3px] border-[var(--border-color)] shadow-[4px_4px_0_var(--neo-shadow)] inline-block font-bold">
          Time elapsed: {(total_time_ms / 1000).toFixed(2)}s
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="neo-card p-6 flex flex-col bg-[var(--neo-primary)] text-[#1a1a1a]">
          <span className="text-sm font-bold uppercase mb-2">Throughput</span>
          <span className="text-4xl font-black">{rps.toFixed(1)}</span>
          <span className="text-xs font-bold uppercase mt-1">req/sec</span>
        </div>
        
        <div className="neo-card p-6 flex flex-col">
          <span className="text-sm font-bold uppercase mb-2">Success Rate</span>
          <span className="text-4xl font-black font-mono text-[var(--neo-accent)]">{successRate}%</span>
          <span className="text-xs font-bold uppercase mt-1 opacity-70">{successes} / {total}</span>
        </div>

        <div className="neo-card p-6 flex flex-col">
          <span className="text-sm font-bold uppercase mb-2">Average (P50)</span>
          <span className="text-4xl font-black font-mono">{p50_latency_ms}ms</span>
          <span className="text-xs font-bold uppercase mt-1 opacity-70">Mean: {avg_latency_ms.toFixed(1)}ms</span>
        </div>

        <div className="neo-card p-6 flex flex-col bg-[var(--neo-danger)] text-[#1a1a1a]">
          <span className="text-sm font-bold uppercase mb-2">Failure Tail (P99)</span>
          <span className="text-4xl font-black font-mono">{p99_latency_ms}ms</span>
          <span className="text-xs font-bold uppercase mt-1">P95: {p95_latency_ms}ms</span>
        </div>
      </div>

      <div className="neo-card bg-[#1a1a1a] text-white p-6 border-b-[8px] border-b-[var(--neo-secondary)] flex flex-col md:flex-row justify-between items-center gap-4">
        <div>
          <h3 className="font-bold text-xl uppercase mb-1 flex items-center gap-2">
            <span className="text-[var(--neo-secondary)]">⚡</span> Engine Synchronization
          </h3>
          <p className="font-mono text-sm opacity-80 max-w-lg">
            Measures internal thundering herd spread. Lower is better. The time difference between the first and last request leaving the reactor.
          </p>
        </div>
        <div className="text-4xl font-black text-[var(--neo-accent)]">
          {spread_us}µs
        </div>
      </div>

      <div className="flex justify-center pt-8">
        <button
          onClick={onReset}
          className="neo-button text-2xl px-12 py-4"
        >
          RUN ANOTHER BARRAGE 🔄
        </button>
      </div>
    </motion.div>
  );
}
