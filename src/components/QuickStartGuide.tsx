import React from 'react';
import { Terminal, ShieldCheck, Zap, Sparkles } from 'lucide-react';

interface Props {
  onOpenBrowse?: () => void;
}

export const QuickStartGuide: React.FC<Props> = ({ onOpenBrowse }) => {
  return (
    <div className="bg-gray-900/60 border border-gray-800 rounded-xl p-5 space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-2 text-white font-semibold text-sm">
          <Sparkles size={16} className="text-yellow-400" />
          <span>Quick Start Guide</span>
        </div>
        {onOpenBrowse && (
          <button
            onClick={onOpenBrowse}
            className="text-xs text-blue-400 hover:text-blue-300 font-medium transition-colors"
          >
            Browse Remote Catalog →
          </button>
        )}
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 text-xs">
        <div className="bg-gray-950 p-3 rounded-lg border border-gray-800 space-y-1">
          <div className="flex items-center space-x-1.5 text-blue-400 font-medium">
            <Zap size={14} />
            <span>1. Install</span>
          </div>
          <p className="text-gray-400 leading-relaxed">
            Select a language and version above or choose from the verified remote catalog.
          </p>
        </div>

        <div className="bg-gray-950 p-3 rounded-lg border border-gray-800 space-y-1">
          <div className="flex items-center space-x-1.5 text-green-400 font-medium">
            <ShieldCheck size={14} />
            <span>2. Zero-Pollution</span>
          </div>
          <p className="text-gray-400 leading-relaxed">
            Runtimes stay isolated in <code className="text-gray-300">~/.devignite/runtimes</code> without polluting PATH.
          </p>
        </div>

        <div className="bg-gray-950 p-3 rounded-lg border border-gray-800 space-y-1">
          <div className="flex items-center space-x-1.5 text-purple-400 font-medium">
            <Terminal size={14} />
            <span>3. Switch & Run</span>
          </div>
          <p className="text-gray-400 leading-relaxed">
            Click activate to route symlinks instantly into your active terminal sessions.
          </p>
        </div>
      </div>
    </div>
  );
};
