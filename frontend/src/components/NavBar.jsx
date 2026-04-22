import React from 'react';
import { useTheme } from '../ThemeContext';
import { Sun, Moon, Zap } from 'lucide-react';

export default function NavBar() {
  const { isDark, toggleTheme } = useTheme();

  return (
    <nav className="w-full border-b-[3px] border-[var(--border-color)] bg-[var(--panel-bg)] p-4 flex justify-between items-center sticky top-0 z-50">
      <div className="flex items-center gap-3">
        <div className="bg-[var(--neo-primary)] p-2 border-[3px] border-[#1a1a1a] shadow-[2px_2px_0px_#1a1a1a]">
          <Zap size={24} color="#1a1a1a" className="fill-[#1a1a1a]" />
        </div>
        <h1 className="text-2xl font-black tracking-tighter uppercase italic">Blitz-Load</h1>
      </div>

      <button
        onClick={toggleTheme}
        className="neo-button p-2"
        aria-label="Toggle theme"
      >
        {isDark ? <Sun size={20} /> : <Moon size={20} />}
      </button>
    </nav>
  );
}
