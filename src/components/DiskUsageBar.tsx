import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { HardDrive } from 'lucide-react';
import { DiskUsageStats } from '../types';

export const DiskUsageBar: React.FC = () => {
  const [stats, setStats] = useState<DiskUsageStats | null>(null);

  const fetchStats = async () => {
    try {
      const data = await invoke<DiskUsageStats>('get_disk_usage_stats');
      setStats(data);
    } catch (e) {
      console.error('Failed to load disk stats', e);
    }
  };

  useEffect(() => {
    fetchStats();
  }, []);

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  if (!stats) return null;

  const total = stats.total_bytes || 1;
  const runtimesPct = Math.max(2, Math.round((stats.runtimes_bytes / total) * 100));
  const cachePct = Math.max(1, Math.round((stats.cache_bytes / total) * 100));
  const dbPct = Math.max(1, 100 - runtimesPct - cachePct);

  return (
    <div className="bg-gray-900 border border-gray-800 p-4 rounded-xl space-y-3">
      <div className="flex items-center justify-between text-xs">
        <div className="flex items-center space-x-2 text-gray-300 font-medium">
          <HardDrive size={15} className="text-blue-400" />
          <span>Storage Breakdown</span>
        </div>
        <span className="text-gray-400 font-mono">{formatBytes(stats.total_bytes)} used</span>
      </div>

      {/* Segmented Progress Bar */}
      <div className="w-full h-2 bg-gray-950 rounded-full overflow-hidden flex">
        <div
          style={{ width: `${runtimesPct}%` }}
          className="bg-blue-500 h-full"
          title={`Runtimes: ${formatBytes(stats.runtimes_bytes)}`}
        />
        <div
          style={{ width: `${cachePct}%` }}
          className="bg-amber-500 h-full"
          title={`Cache: ${formatBytes(stats.cache_bytes)}`}
        />
        <div
          style={{ width: `${dbPct}%` }}
          className="bg-purple-500 h-full"
          title={`Database: ${formatBytes(stats.db_bytes)}`}
        />
      </div>

      <div className="flex justify-between items-center text-[11px] text-gray-400 font-mono">
        <div className="flex items-center space-x-1.5">
          <span className="w-2 h-2 rounded-full bg-blue-500 inline-block" />
          <span>Runtimes: {formatBytes(stats.runtimes_bytes)}</span>
        </div>
        <div className="flex items-center space-x-1.5">
          <span className="w-2 h-2 rounded-full bg-amber-500 inline-block" />
          <span>Cache: {formatBytes(stats.cache_bytes)}</span>
        </div>
        <div className="flex items-center space-x-1.5">
          <span className="w-2 h-2 rounded-full bg-purple-500 inline-block" />
          <span>DB: {formatBytes(stats.db_bytes)}</span>
        </div>
      </div>
    </div>
  );
};
