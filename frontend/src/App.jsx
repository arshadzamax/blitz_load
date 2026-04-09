import React, { useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { runLoadTest } from './api';
import NavBar from './components/NavBar';
import Hero from './components/Hero';
import ConfigPanel from './components/ConfigPanel';
import LiveDashboard from './components/LiveDashboard';
import ResultsSummary from './components/ResultsSummary';

const VIEW = {
  HERO: 'hero',
  CONFIG: 'config',
  LIVE: 'live',
  RESULTS: 'results'
};

export default function App() {
  const [currentView, setCurrentView] = useState(VIEW.HERO);
  const [testId, setTestId] = useState(null);
  const [results, setResults] = useState(null);
  const [isStarting, setIsStarting] = useState(false);
  const [errorStr, setErrorStr] = useState('');

  const handleStartConfig = () => {
    setCurrentView(VIEW.CONFIG);
  };

  const handleLaunchTest = async (config) => {
    try {
      setIsStarting(true);
      setErrorStr('');
      const res = await runLoadTest(config);
      setTestId(res.test_id);
      setIsStarting(false);
      setCurrentView(VIEW.LIVE);
    } catch (err) {
      setErrorStr(err.message || 'Error communicating with base.');
      setIsStarting(false);
    }
  };

  const handleTestComplete = (finalResults) => {
    setResults(finalResults);
    setCurrentView(VIEW.RESULTS);
  };

  const handleTestError = (msg) => {
    setErrorStr(`Test failed: ${msg}`);
    setCurrentView(VIEW.CONFIG);
  };

  const handleReset = () => {
    setTestId(null);
    setResults(null);
    setErrorStr('');
    setCurrentView(VIEW.CONFIG);
  };

  return (
    <div className="min-h-screen flex flex-col font-sans">
      <NavBar />
      
      <main className="flex-1 overflow-x-hidden p-4 relative pt-12">
        {errorStr && (
          <div className="max-w-xl mx-auto mb-6 bg-[var(--neo-danger)] text-[#1a1a1a] p-4 border-[3px] border-[#1a1a1a] shadow-[4px_4px_0_var(--neo-shadow)] font-bold flex justify-between">
            <span>⚠️ {errorStr}</span>
            <button onClick={() => setErrorStr('')} className="underline uppercase text-xs">Dismiss</button>
          </div>
        )}

        <AnimatePresence mode="wait">
          {currentView === VIEW.HERO && (
            <motion.div key="hero" exit={{ opacity: 0, y: -20 }} transition={{ duration: 0.2 }}>
              <Hero onStart={handleStartConfig} />
            </motion.div>
          )}

          {currentView === VIEW.CONFIG && (
            <motion.div key="config" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.2 }}>
              <ConfigPanel onLaunch={handleLaunchTest} isLoading={isStarting} />
            </motion.div>
          )}

          {currentView === VIEW.LIVE && testId && (
            <motion.div key="live" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.2 }}>
              <LiveDashboard testId={testId} onComplete={handleTestComplete} onError={handleTestError} />
            </motion.div>
          )}

          {currentView === VIEW.RESULTS && results && (
            <motion.div key="results" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.2 }}>
              <ResultsSummary results={results} onReset={handleReset} />
            </motion.div>
          )}
        </AnimatePresence>
      </main>
      
      <footer className="p-6 text-center text-sm font-bold uppercase opacity-50 border-t-[3px] border-[var(--border-color)]">
        Powered by Rust 🦀 &middot; Web Dashboard 🌐
      </footer>
    </div>
  );
}
