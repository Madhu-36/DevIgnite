import React, { useState } from 'react';
import { Play, Trash2, CheckCircle2, Circle, Clock, Folder } from 'lucide-react';
import { Runtime } from '../types';
import { useDevIgniteStore } from '../store/useDevIgniteStore';
import { ConfirmDialog } from './ConfirmDialog';

interface Props {
  runtime: Runtime;
}

const TOOL_COLORS: Record<string, string> = {
  node: 'text-green-400 bg-green-500/10 border-green-500/30',
  python: 'text-yellow-400 bg-yellow-500/10 border-yellow-500/30',
  go: 'text-cyan-400 bg-cyan-500/10 border-cyan-500/30',
  rust: 'text-orange-400 bg-orange-500/10 border-orange-500/30',
};

export const RuntimeCard: React.FC<Props> = ({ runtime }) => {
  const switchVersion = useDevIgniteStore((state) => state.switchVersion);
  const uninstallRuntime = useDevIgniteStore((state) => state.uninstallRuntime);
  const isLoading = useDevIgniteStore((state) => state.isLoading);
  const [showConfirm, setShowConfirm] = useState(false);

  const badgeColor = TOOL_COLORS[runtime.tool_name.toLowerCase()] || 'text-blue-400 bg-blue-500/10 border-blue-500/30';

  const handleUninstall = () => {
    setShowConfirm(false);
    uninstallRuntime(runtime.tool_name, runtime.version);
  };

  return (
    <>
      <div className="bg-gray-800 border border-gray-700 hover:border-gray-600 transition-all rounded-xl p-4 flex items-center justify-between text-white shadow-sm">
        <div className="flex items-center space-x-4">
          {runtime.is_active ? (
            <CheckCircle2 className="text-green-400 w-6 h-6 shrink-0" />
          ) : (
            <Circle className="text-gray-500 w-6 h-6 shrink-0" />
          )}

          <div>
            <div className="flex items-center space-x-2">
              <span className={`text-xs px-2 py-0.5 rounded-full border font-mono font-medium ${badgeColor}`}>
                {runtime.tool_name.toUpperCase()}
              </span>
              <h3 className="text-base font-semibold">v{runtime.version}</h3>
              {runtime.is_active && (
                <span className="text-[10px] bg-green-500/20 text-green-300 px-2 py-0.5 rounded-full font-medium">
                  Active in PATH
                </span>
              )}
            </div>

            <div className="flex items-center space-x-3 text-xs text-gray-400 mt-1">
              <span className="flex items-center space-x-1" title={runtime.path}>
                <Folder size={12} />
                <span className="truncate max-w-[180px]">{runtime.path}</span>
              </span>
              {runtime.installed_at && (
                <span className="flex items-center space-x-1">
                  <Clock size={12} />
                  <span>{new Date(runtime.installed_at).toLocaleDateString()}</span>
                </span>
              )}
            </div>
          </div>
        </div>

        <div className="flex items-center space-x-2">
          {!runtime.is_active && (
            <button
              onClick={() => switchVersion(runtime.tool_name, runtime.version)}
              disabled={isLoading}
              className="p-2 hover:bg-gray-700 rounded-lg transition-colors text-blue-400 hover:text-blue-300 disabled:opacity-50"
              title="Activate this runtime"
            >
              <Play className="w-5 h-5" />
            </button>
          )}

          <button
            onClick={() => setShowConfirm(true)}
            disabled={isLoading}
            className="p-2 hover:bg-red-900/40 rounded-lg transition-colors text-red-400 hover:text-red-300 disabled:opacity-50"
            title="Uninstall runtime"
          >
            <Trash2 className="w-5 h-5" />
          </button>
        </div>
      </div>

      <ConfirmDialog
        isOpen={showConfirm}
        title={`Uninstall ${runtime.tool_name} v${runtime.version}`}
        message={`Are you sure you want to completely remove ${runtime.tool_name} v${runtime.version} from your system? Its storage directory will be deleted.`}
        confirmLabel="Uninstall"
        isDestructive={true}
        onConfirm={handleUninstall}
        onCancel={() => setShowConfirm(false)}
      />
    </>
  );
};
