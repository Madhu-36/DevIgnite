import React, { useEffect, useState } from 'react';
import { Modal } from './Modal';
import { invoke } from '@tauri-apps/api/core';
import { History, Clock, ArrowRight } from 'lucide-react';
import { SwitchHistoryEntry } from '../types';

interface Props {
  isOpen: boolean;
  onClose: () => void;
}

export const UsageHistoryModal: React.FC<Props> = ({ isOpen, onClose }) => {
  const [history, setHistory] = useState<SwitchHistoryEntry[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (isOpen) {
      setLoading(true);
      invoke<SwitchHistoryEntry[]>('get_switch_history', { limit: 25 })
        .then((data) => setHistory(data))
        .catch((e) => console.error(e))
        .finally(() => setLoading(false));
    }
  }, [isOpen]);

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Runtime Switch Audit Log" maxWidth="max-w-md">
      <div className="space-y-4">
        {loading ? (
          <div className="p-8 text-center text-xs text-gray-400">Loading audit history...</div>
        ) : history.length === 0 ? (
          <div className="p-8 text-center text-xs text-gray-400">
            No runtime switches recorded yet.
          </div>
        ) : (
          <div className="space-y-2 max-h-80 overflow-y-auto pr-1">
            {history.map((item) => (
              <div
                key={item.id}
                className="bg-gray-900 border border-gray-800 p-3 rounded-lg flex items-center justify-between"
              >
                <div className="flex items-center space-x-2">
                  <div className="p-1.5 bg-blue-500/10 text-blue-400 rounded-md">
                    <History size={14} />
                  </div>
                  <div>
                    <div className="flex items-center space-x-1.5 text-xs text-white font-medium">
                      <span className="capitalize">{item.tool_name}</span>
                      <ArrowRight size={12} className="text-gray-500" />
                      <span className="font-mono text-blue-300">v{item.version}</span>
                    </div>
                    <div className="flex items-center space-x-1 text-[10px] text-gray-500 font-mono mt-0.5">
                      <Clock size={10} />
                      <span>{new Date(item.switched_at).toLocaleString()}</span>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </Modal>
  );
};
