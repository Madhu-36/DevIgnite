import React from 'react';
import { Play, Trash2, CheckCircle2, Circle } from 'lucide-react';
import { Runtime, useDevIgniteStore } from '../store/useDevIgniteStore';

interface Props {
  runtime: Runtime;
}

export const RuntimeCard: React.FC<Props> = ({ runtime }) => {
  const switchVersion = useDevIgniteStore((state) => state.switchVersion);
  const uninstallRuntime = useDevIgniteStore((state) => state.uninstallRuntime);

  return (
    <div className="bg-gray-800 border border-gray-700 rounded-lg p-4 flex items-center justify-between text-white shadow-sm">
      <div className="flex items-center space-x-4">
        {runtime.is_active ? (
          <CheckCircle2 className="text-green-400 w-6 h-6" />
        ) : (
          <Circle className="text-gray-500 w-6 h-6" />
        )}
        <div>
          <h3 className="text-lg font-semibold capitalize">{runtime.tool_name}</h3>
          <p className="text-sm text-gray-400">v{runtime.version}</p>
        </div>
      </div>

      <div className="flex items-center space-x-2">
        {!runtime.is_active && (
          <button
            onClick={() => switchVersion(runtime.tool_name, runtime.version)}
            className="p-2 hover:bg-gray-700 rounded-md transition-colors text-blue-400"
            title="Switch to this version"
          >
            <Play className="w-5 h-5" />
          </button>
        )}
        <button
          onClick={() => uninstallRuntime(runtime.tool_name, runtime.version)}
          className="p-2 hover:bg-red-900/50 rounded-md transition-colors text-red-400"
          title="Uninstall"
        >
          <Trash2 className="w-5 h-5" />
        </button>
      </div>
    </div>
  );
};
