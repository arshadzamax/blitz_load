import React from 'react';
import { motion } from 'framer-motion';

export default function Hero({ onStart }) {
  return (
    <div className="flex flex-col items-center justify-center min-h-[70vh] px-4 text-center">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="max-w-3xl border-[3px] border-[var(--border-color)] bg-[var(--panel-bg)] p-8 md:p-16 shadow-[8px_8px_0_var(--neo-shadow)] relative"
      >
        <div className="absolute top-0 left-0 w-full h-2 bg-[var(--neo-primary)] border-b-[3px] border-[var(--border-color)]" />
        
        <h2 className="text-4xl md:text-6xl font-black uppercase mb-6 leading-tight">
          Break your API <br /> 
          <span className="text-transparent bg-clip-text bg-gradient-to-r from-[var(--neo-secondary)] to-[var(--neo-danger)] drop-shadow-[2px_2px_0_var(--text-color)]">
            before users do.
          </span>
        </h2>
        
        <p className="text-lg md:text-xl font-medium mb-12 max-w-2xl mx-auto opacity-80 font-mono">
          High-performance hybrid load testing. Raw Rust muscle. Real-time metrics.
        </p>

        <motion.button
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.95 }}
          onClick={onStart}
          className="neo-button neo-button-primary text-xl md:text-2xl px-8 py-4 mx-auto"
        >
          START A TEST 🚀
        </motion.button>
      </motion.div>
    </div>
  );
}
