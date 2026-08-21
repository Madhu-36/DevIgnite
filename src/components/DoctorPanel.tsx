import React, { useEffect } from 'react';
import { Activity, AlertTriangle, CheckCircle, ShieldAlert } from 'lucide-react';
import { useDevIgniteStore } from '../store/useDevIgniteStore';

export const DoctorPanel: React.FC = () => {
  const { diagnostics, runDiagnostics, isLoading } = useDevIgniteStore();

  useEffect(() => {
    runDiagnostics();
  }, [runDiagnostics]);

  if (!diagnostics) {
    return (
      <div className="bg-gray-800 p-6 rounded-lg text-white flex items-center justify-center">
        <Activity className="animate-spin w-6 h-6 mr-3 text-blue-400" />
        <span>Running diagnostics...</span>
      </div>
    );
  }

  return (
    <div className="bg-gray-800 rounded-lg p-6 text-white shadow-md">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-bold flex items-center">
          <Activity className="w-6 h-6 mr-2 text-blue-400" />
          DevIgnite Doctor
        </h2>
        <button
          onClick={() => runDiagnostics()}
          disabled={isLoading}
          className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-md text-sm font-medium transition-colors disabled:opacity-50"
        >
          Refresh
        </button>
      </div>

      {diagnostics.is_healthy ? (
        <div className="flex items-center text-green-400 bg-green-900/20 p-4 rounded-md">
          <CheckCircle className="w-5 h-5 mr-3" />
          <p>System is perfectly healthy. No PATH collisions or broken symlinks detected.</p>
        </div>
      ) : (
        <div className="space-y-4">
          {diagnostics.path_collisions.length > 0 && (
            <div className="bg-orange-900/20 border border-orange-800 rounded-md p-4">
              <h3 className="text-orange-400 font-semibold flex items-center mb-2">
                <AlertTriangle className="w-5 h-5 mr-2" />
                PATH Collisions Detected
              </h3>
              <ul className="list-disc list-inside text-sm text-gray-300">
                {diagnostics.path_collisions.map((collision, idx) => (
                  <li key={idx}>{collision}</li>
                ))}
              </ul>
            </div>
          )}

          {diagnostics.broken_symlinks.length > 0 && (
            <div className="bg-red-900/20 border border-red-800 rounded-md p-4">
              <h3 className="text-red-400 font-semibold flex items-center mb-2">
                <ShieldAlert className="w-5 h-5 mr-2" />
                Broken Symlinks Detected
              </h3>
              <ul className="list-disc list-inside text-sm text-gray-300">
                {diagnostics.broken_symlinks.map((link, idx) => (
                  <li key={idx}>{link}</li>
                ))}
              </ul>
            </div>
          )}

          {diagnostics.permission_issues.length > 0 && (
            <div className="bg-yellow-900/20 border border-yellow-800 rounded-md p-4">
              <h3 className="text-yellow-400 font-semibold flex items-center mb-2">
                <AlertTriangle className="w-5 h-5 mr-2" />
                Permission Issues
              </h3>
              <ul className="list-disc list-inside text-sm text-gray-300">
                {diagnostics.permission_issues.map((issue, idx) => (
                  <li key={idx}>{issue}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
