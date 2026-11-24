import React, { useState, useEffect } from 'react';
import { Modal } from './Modal';
import { invoke } from '@tauri-apps/api/core';
import { Settings, Trash2, Download, Save } from 'lucide-react';
import { useDevIgniteStore } from '../store/useDevIgniteStore';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({ isOpen, onClose }) => {
  const { clearCache } = useDevIgniteStore();
  const [autoDoctor, setAutoDoctor] = useState(true);
  const [strictChecksum, setStrictChecksum] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [exportedJson, setExportedJson] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen) {
      invoke<string>('get_system_config', { key: 'auto_doctor' })
        .then((val) => setAutoDoctor(val !== 'false'))
        .catch(() => {});
      invoke<string>('get_system_config', { key: 'strict_checksum' })
        .then((val) => setStrictChecksum(val !== 'false'))
        .catch(() => {});
    }
  }, [isOpen]);

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await invoke('set_system_config', { key: 'auto_doctor', value: String(autoDoctor) });
      await invoke('set_system_config', { key: 'strict_checksum', value: String(strictChecksum) });
      onClose();
    } catch (e) {
      console.error('Failed to save config:', e);
    } finally {
      setIsSaving(false);
    }
  };

  const handleExport = async () => {
    try {
      const data = await invoke<string>('export_runtimes_config');
      setExportedJson(data);
    } catch (e) {
      console.error('Failed to export config:', e);
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="DevIgnite Preferences & Settings" maxWidth="max-w-lg">
      <div className="space-y-6">
        <div className="space-y-4">
          <label className="flex items-center justify-between p-3 bg-gray-900/60 rounded-lg border border-gray-700 cursor-pointer">
            <div>
              <p className="text-sm font-medium text-white">Auto-Run Health Doctor</p>
              <p className="text-xs text-gray-400">Run diagnostics automatically when launching the dashboard</p>
            </div>
            <input
              type="checkbox"
              checked={autoDoctor}
              onChange={(e) => setAutoDoctor(e.target.checked)}
              className="w-4 h-4 text-blue-600 rounded bg-gray-700 border-gray-600 focus:ring-blue-500"
            />
          </label>

          <label className="flex items-center justify-between p-3 bg-gray-900/60 rounded-lg border border-gray-700 cursor-pointer">
            <div>
              <p className="text-sm font-medium text-white">Strict SHA-256 Verification</p>
              <p className="text-xs text-gray-400">Reject downloads if expected checksum is missing or mismatched</p>
            </div>
            <input
              type="checkbox"
              checked={strictChecksum}
              onChange={(e) => setStrictChecksum(e.target.checked)}
              className="w-4 h-4 text-blue-600 rounded bg-gray-700 border-gray-600 focus:ring-blue-500"
            />
          </label>
        </div>

        <div className="pt-4 border-t border-gray-700 flex justify-between items-center">
          <button
            onClick={() => clearCache()}
            className="flex items-center space-x-2 text-xs text-red-400 hover:text-red-300 py-1 px-2 rounded hover:bg-red-500/10 transition-colors"
          >
            <Trash2 size={14} />
            <span>Purge Cache</span>
          </button>

          <button
            onClick={handleExport}
            className="flex items-center space-x-2 text-xs text-gray-300 hover:text-white py-1 px-2 rounded hover:bg-gray-700 transition-colors"
          >
            <Download size={14} />
            <span>Export Manifest</span>
          </button>
        </div>

        {exportedJson && (
          <div className="mt-2">
            <p className="text-xs text-gray-400 mb-1">Configuration JSON:</p>
            <textarea
              readOnly
              value={exportedJson}
              className="w-full h-24 p-2 text-xs bg-gray-950 font-mono text-green-400 rounded border border-gray-700 resize-none"
            />
          </div>
        )}

        <div className="flex justify-end space-x-3 pt-4 border-t border-gray-700">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm text-gray-300 hover:bg-gray-700 rounded-lg transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={isSaving}
            className="flex items-center space-x-2 px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-500 rounded-lg transition-colors disabled:opacity-50"
          >
            <Save size={16} />
            <span>{isSaving ? 'Saving...' : 'Save Settings'}</span>
          </button>
        </div>
      </div>
    </Modal>
  );
};
