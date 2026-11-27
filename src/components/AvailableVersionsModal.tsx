import React, { useEffect, useState } from 'react';
import { Modal } from './Modal';
import { invoke } from '@tauri-apps/api/core';
import { DownloadCloud, Tag, Calendar, Plus } from 'lucide-react';
import { RemoteVersion } from '../types';
import { useDevIgniteStore } from '../store/useDevIgniteStore';

interface Props {
  isOpen: boolean;
  onClose: () => void;
  onInstalled?: (tool: string, version: string) => void;
}

export const AvailableVersionsModal: React.FC<Props> = ({ isOpen, onClose, onInstalled }) => {
  const [versions, setVersions] = useState<RemoteVersion[]>([]);
  const [activeTab, setActiveTab] = useState('node');
  const [loading, setLoading] = useState(false);
  const { installRuntime, isLoading } = useDevIgniteStore();

  useEffect(() => {
    if (isOpen) {
      setLoading(true);
      invoke<RemoteVersion[]>('list_available_remote_versions', { tool: activeTab })
        .then((data) => setVersions(data))
        .catch((err) => console.error(err))
        .finally(() => setLoading(false));
    }
  }, [isOpen, activeTab]);

  const handleQuickInstall = async (item: RemoteVersion) => {
    try {
      await installRuntime(item.tool_name, item.version);
      if (onInstalled) {
        onInstalled(item.tool_name, item.version);
      }
      onClose();
    } catch (e) {
      console.error('Install error:', e);
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Browse Remote Runtime Catalog" maxWidth="max-w-2xl">
      <div className="space-y-4">
        {/* Language Tabs */}
        <div className="flex space-x-2 border-b border-gray-700 pb-2">
          {['node', 'python', 'go', 'rust'].map((tool) => (
            <button
              key={tool}
              onClick={() => setActiveTab(tool)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium capitalize transition-colors ${
                activeTab === tool
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-400 hover:text-white hover:bg-gray-800'
              }`}
            >
              {tool}
            </button>
          ))}
        </div>

        {/* Version List */}
        <div className="space-y-2 max-h-80 overflow-y-auto pr-1">
          {loading ? (
            <div className="p-8 text-center text-xs text-gray-400">Loading catalog...</div>
          ) : versions.length === 0 ? (
            <div className="p-8 text-center text-xs text-gray-400">No versions available.</div>
          ) : (
            versions.map((v) => (
              <div
                key={`${v.tool_name}-${v.version}`}
                className="bg-gray-900 border border-gray-800 p-3 rounded-lg flex items-center justify-between"
              >
                <div className="space-y-1">
                  <div className="flex items-center space-x-2">
                    <span className="text-sm font-semibold text-white">v{v.version}</span>
                    {v.lts && (
                      <span className="flex items-center space-x-1 text-[10px] bg-green-500/10 text-green-400 border border-green-500/30 px-2 py-0.5 rounded-full font-medium">
                        <Tag size={10} />
                        <span>{v.lts}</span>
                      </span>
                    )}
                  </div>
                  {v.release_date && (
                    <div className="flex items-center space-x-1 text-xs text-gray-500 font-mono">
                      <Calendar size={11} />
                      <span>{v.release_date}</span>
                    </div>
                  )}
                </div>

                <button
                  onClick={() => handleQuickInstall(v)}
                  disabled={isLoading}
                  className="flex items-center space-x-1 px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white text-xs font-medium rounded-md transition-colors disabled:opacity-50"
                >
                  <Plus size={14} />
                  <span>Install</span>
                </button>
              </div>
            ))
          )}
        </div>
      </div>
    </Modal>
  );
};
