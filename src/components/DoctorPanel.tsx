import React, { useEffect, useState } from 'react';
import { Activity, CheckCircle, ShieldAlert, Wrench, RefreshCw, Check, X } from 'lucide-react';
import { useDevIgniteStore } from '../store/useDevIgniteStore';
import { invoke } from '@tauri-apps/api/core';
import { AutoFixResult } from '../types';

export const DoctorPanel: React.FC = () => {
  const { diagnostics, runDiagnostics, isLoading } = useDevIgniteStore();
  const [isFixing, setIsFixing] = useState(false);
  const [fixResult, setFixResult] = useState<AutoFixResult | null>(null);

  useEffect(() => {
    runDiagnostics();
  }, [runDiagnostics]);

  const handleAutoFix = async () => {
    setIsFixing(true);
    setFixResult(null);
    try {
      const res = await invoke<AutoFixResult>('auto_fix_diagnostics_cmd');
      setFixResult(res);
      await runDiagnostics();
    } catch (e) {
      console.error('Auto-fix failed:', e);
    } finally {
      setIsFixing(false);
    }
  };

  if (!diagnostics) {
    return (
      <div className="bg-gray-800 p-6 rounded-lg text-white flex items-center justify-center border border-gray-700">
        <Activity className="animate-spin w-6 h-6 mr-3 text-blue-400" />
        <span>Running diagnostics...</span>
      </div>
    );
  }

  return (
    <div className="bg-gray-800 rounded-lg p-6 text-white shadow-md border border-gray-700">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-bold flex items-center">
          <Activity className="w-6 h-6 mr-2 text-blue-400" />
          DevIgnite Doctor Diagnostics
        </h2>
        <div className="flex items-center space-x-2">
          {diagnostics.auto_fix_available && (
            <button
              onClick={handleAutoFix}
              disabled={isFixing || isLoading}
              className="flex items-center space-x-1 px-3 py-1.5 bg-yellow-600 hover:bg-yellow-500 rounded-md text-xs font-semibold text-white transition-colors disabled:opacity-50"
            >
              <Wrench size={14} />
              <span>{isFixing ? 'Fixing...' : 'Auto-Fix Issues'}</span>
            </button>
          )}
          <button
            onClick={() => runDiagnostics()}
            disabled={isLoading || isFixing}
            className="flex items-center space-x-1 px-3 py-1.5 bg-gray-700 hover:bg-gray-600 rounded-md text-xs font-medium text-gray-200 transition-colors disabled:opacity-50"
          >
            <RefreshCw size={14} className={isLoading ? 'animate-spin' : ''} />
            <span>Rescan</span>
          </button>
        </div>
      </div>

      {fixResult && (
        <div
          className={`mb-4 p-3 rounded-lg text-xs ${
            fixResult.success ? 'bg-green-950/60 border border-green-800 text-green-300' : 'bg-red-950/60 border border-red-800 text-red-300'
          }`}
        >
          <p className="font-semibold">{fixResult.success ? 'Auto-fix completed successfully!' : 'Auto-fix encountered issues.'}</p>
          {fixResult.fixed_items.length > 0 && (
            <p className="mt-1">Removed dangling items: {fixResult.fixed_items.join(', ')}</p>
          )}
          {fixResult.remaining_errors.length > 0 && (
            <p className="mt-1 text-red-400">Errors: {fixResult.remaining_errors.join(', ')}</p>
          )}
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-3 gap-3 mb-4">
        <div className="bg-gray-900/60 p-3 rounded-lg border border-gray-700 flex items-center justify-between">
          <div>
            <p className="text-xs text-gray-400">Symlink Integrity</p>
            <p className="text-sm font-semibold">{diagnostics.symlink_healthy ? 'Healthy' : 'Dangling Links'}</p>
          </div>
          {diagnostics.symlink_healthy ? (
            <Check className="text-green-400" size={18} />
          ) : (
            <X className="text-red-400" size={18} />
          )}
        </div>

        <div className="bg-gray-900/60 p-3 rounded-lg border border-gray-700 flex items-center justify-between">
          <div>
            <p className="text-xs text-gray-400">PATH Registration</p>
            <p className="text-sm font-semibold">{diagnostics.devignite_in_path ? 'Configured' : 'Missing from PATH'}</p>
          </div>
          {diagnostics.devignite_in_path ? (
            <Check className="text-green-400" size={18} />
          ) : (
            <X className="text-yellow-400" size={18} />
          )}
        </div>

        <div className="bg-gray-900/60 p-3 rounded-lg border border-gray-700 flex items-center justify-between">
          <div>
            <p className="text-xs text-gray-400">File Permissions</p>
            <p className="text-sm font-semibold">{diagnostics.permissions_ok ? 'Write Permitted' : 'Permission Denied'}</p>
          </div>
          {diagnostics.permissions_ok ? (
            <Check className="text-green-400" size={18} />
          ) : (
            <X className="text-red-400" size={18} />
          )}
        </div>
      </div>

      {diagnostics.is_healthy ? (
        <div className="flex items-center text-green-400 bg-green-900/20 border border-green-800/40 p-4 rounded-md">
          <CheckCircle className="w-5 h-5 mr-3 shrink-0" />
          <p className="text-sm">Sandbox environment is optimal. All symlinks point to verified runtime paths.</p>
        </div>
      ) : (
        <div className="space-y-3">
          {diagnostics.error_details.length > 0 && (
            <div className="bg-red-950/40 border border-red-800/50 rounded-md p-4">
              <h3 className="text-red-400 font-semibold flex items-center mb-2 text-sm">
                <ShieldAlert className="w-4 h-4 mr-2" />
                Diagnostic Warnings
              </h3>
              <ul className="list-disc list-inside text-xs text-gray-300 space-y-1">
                {diagnostics.error_details.map((detail, idx) => (
                  <li key={idx}>{detail}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
