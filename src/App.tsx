import { useEffect, useState, useMemo } from 'react';
import { useDevIgniteStore } from './store/useDevIgniteStore';
import { RuntimeCard } from './components/RuntimeCard';
import { DoctorPanel } from './components/DoctorPanel';
import { SearchBar } from './components/SearchBar';
import { SettingsModal } from './components/SettingsModal';
import { AvailableVersionsModal } from './components/AvailableVersionsModal';
import { UsageHistoryModal } from './components/UsageHistoryModal';
import { DiskUsageBar } from './components/DiskUsageBar';
import { QuickStartGuide } from './components/QuickStartGuide';
import { ToastContainer } from './components/Toast';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';
import { Layers, Settings, RefreshCw, PlusCircle, AlertCircle, Globe, History } from 'lucide-react';
import { ToastMessage } from './types';

function App() {
  const { runtimes, fetchRuntimes, installRuntime, isLoading, errorMessage, setErrorMessage } =
    useDevIgniteStore();

  const [searchQuery, setSearchQuery] = useState('');
  const [selectedTool, setSelectedTool] = useState('all');
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isCatalogOpen, setIsCatalogOpen] = useState(false);
  const [isHistoryOpen, setIsHistoryOpen] = useState(false);
  const [toasts, setToasts] = useState<ToastMessage[]>([]);

  const [inputTool, setInputTool] = useState('');
  const [inputVersion, setInputVersion] = useState('');

  const addToast = (message: string, type: 'success' | 'error' | 'info' = 'info') => {
    const id = Date.now().toString();
    setToasts((prev) => [...prev, { id, message, type }]);
  };

  const removeToast = (id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  };

  useEffect(() => {
    fetchRuntimes();
  }, [fetchRuntimes]);

  useKeyboardShortcuts({
    onRefresh: () => {
      fetchRuntimes();
      addToast('Runtimes refreshed', 'info');
    },
    onOpenSettings: () => setIsSettingsOpen(true),
  });

  const filteredRuntimes = useMemo(() => {
    return runtimes.filter((rt) => {
      const matchesTool =
        selectedTool === 'all' || rt.tool_name.toLowerCase() === selectedTool.toLowerCase();
      const matchesSearch =
        rt.tool_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        rt.version.toLowerCase().includes(searchQuery.toLowerCase());
      return matchesTool && matchesSearch;
    });
  }, [runtimes, selectedTool, searchQuery]);

  const handleInstall = async () => {
    if (!inputTool || !inputVersion) return;
    try {
      await installRuntime(inputTool.toLowerCase(), inputVersion);
      addToast(`Installed ${inputTool} v${inputVersion}`, 'success');
      setInputTool('');
      setInputVersion('');
    } catch {
      addToast(`Installation failed for ${inputTool}`, 'error');
    }
  };

  return (
    <div className="min-h-screen bg-gray-950 text-gray-100 p-6 md:p-10 font-sans selection:bg-blue-500 selection:text-white">
      <div className="max-w-6xl mx-auto space-y-8">
        {/* Header */}
        <header className="flex items-center justify-between border-b border-gray-800 pb-6">
          <div className="flex items-center space-x-3">
            <div className="p-2 bg-blue-600/20 text-blue-400 rounded-xl border border-blue-500/30">
              <Layers className="w-7 h-7" />
            </div>
            <div>
              <h1 className="text-2xl font-bold tracking-tight text-white flex items-center space-x-2">
                <span>DevIgnite</span>
                <span className="text-xs px-2 py-0.5 bg-blue-900/50 text-blue-300 border border-blue-800 rounded-full font-mono">
                  v0.3.0
                </span>
              </h1>
              <p className="text-xs text-gray-400">Sandboxed Symlink Runtime Engine</p>
            </div>
          </div>

          <div className="flex items-center space-x-2 sm:space-x-3">
            <button
              onClick={() => setIsCatalogOpen(true)}
              className="flex items-center space-x-1.5 px-3 py-2 text-xs font-medium text-gray-300 hover:text-white bg-gray-900 hover:bg-gray-800 border border-gray-800 rounded-lg transition-colors"
              title="Remote Catalog"
            >
              <Globe size={15} className="text-blue-400" />
              <span className="hidden sm:inline">Browse Online</span>
            </button>

            <button
              onClick={() => setIsHistoryOpen(true)}
              className="flex items-center space-x-1.5 px-3 py-2 text-xs font-medium text-gray-300 hover:text-white bg-gray-900 hover:bg-gray-800 border border-gray-800 rounded-lg transition-colors"
              title="Audit Log"
            >
              <History size={15} className="text-purple-400" />
              <span className="hidden sm:inline">Switch Log</span>
            </button>

            <button
              onClick={() => {
                fetchRuntimes();
                addToast('Refreshing runtimes...', 'info');
              }}
              disabled={isLoading}
              className="p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded-lg transition-colors border border-gray-800 disabled:opacity-50"
              title="Refresh (Ctrl+R)"
            >
              <RefreshCw size={17} className={isLoading ? 'animate-spin' : ''} />
            </button>

            <button
              onClick={() => setIsSettingsOpen(true)}
              className="flex items-center space-x-1.5 px-3 py-2 text-xs font-medium text-gray-300 hover:text-white bg-gray-800/80 hover:bg-gray-800 border border-gray-700 rounded-lg transition-colors"
              title="Preferences (Ctrl+,)"
            >
              <Settings size={15} />
              <span>Settings</span>
            </button>
          </div>
        </header>

        {/* Global Error Banner */}
        {errorMessage && (
          <div className="bg-red-950/60 border border-red-800/80 text-red-200 px-4 py-3 rounded-xl flex justify-between items-center animate-fadeIn shadow-lg">
            <div className="flex items-center space-x-3">
              <AlertCircle size={20} className="text-red-400 shrink-0" />
              <span className="text-sm">{errorMessage}</span>
            </div>
            <button
              onClick={() => setErrorMessage(null)}
              className="text-red-400 hover:text-red-200 p-1 text-sm font-bold"
            >
              ✕
            </button>
          </div>
        )}

        {/* Main Workspace */}
        <main className="grid grid-cols-1 lg:grid-cols-12 gap-8">
          {/* Left Column: Runtimes & Installer */}
          <div className="lg:col-span-7 space-y-6">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-semibold text-gray-200">Installed Runtimes</h2>
              <span className="text-xs text-gray-500 font-mono">
                {filteredRuntimes.length} of {runtimes.length} tool(s)
              </span>
            </div>

            {/* Quick Install Bar */}
            <div className="bg-gray-900 border border-gray-800 p-4 rounded-xl shadow-sm space-y-3">
              <div className="flex items-center justify-between">
                <p className="text-xs font-semibold text-gray-400 uppercase tracking-wider">
                  Install New Runtime
                </p>
                <button
                  onClick={() => setIsCatalogOpen(true)}
                  className="text-xs text-blue-400 hover:text-blue-300 font-medium"
                >
                  From Remote Catalog →
                </button>
              </div>
              <div className="flex space-x-2">
                <input
                  type="text"
                  placeholder="Tool (e.g. node, python, go, rust)"
                  value={inputTool}
                  onChange={(e) => setInputTool(e.target.value)}
                  disabled={isLoading}
                  className="bg-gray-950 border border-gray-700 rounded-lg px-3 py-2 text-xs text-white focus:outline-none focus:border-blue-500 w-1/2 disabled:opacity-50"
                />
                <input
                  type="text"
                  placeholder="Version (e.g. 20.11.0)"
                  value={inputVersion}
                  onChange={(e) => setInputVersion(e.target.value)}
                  disabled={isLoading}
                  className="bg-gray-950 border border-gray-700 rounded-lg px-3 py-2 text-xs text-white focus:outline-none focus:border-blue-500 w-1/3 disabled:opacity-50"
                />
                <button
                  onClick={handleInstall}
                  disabled={isLoading || !inputTool || !inputVersion}
                  className="px-4 py-2 bg-blue-600 hover:bg-blue-500 rounded-lg text-xs font-medium text-white transition-colors disabled:opacity-50 flex items-center justify-center space-x-1.5 shrink-0"
                >
                  <PlusCircle size={15} />
                  <span>{isLoading ? 'Installing...' : 'Install'}</span>
                </button>
              </div>
            </div>

            {/* Search and Filters */}
            <SearchBar
              searchQuery={searchQuery}
              onSearchChange={setSearchQuery}
              selectedTool={selectedTool}
              onToolSelect={setSelectedTool}
            />

            {/* Runtime Cards List */}
            <div>
              {filteredRuntimes.length === 0 ? (
                <div className="space-y-4">
                  <div className="bg-gray-900/60 border border-gray-800/80 rounded-xl p-8 text-center text-gray-400 space-y-2">
                    <p className="text-sm font-medium">No matching runtimes found.</p>
                    <p className="text-xs text-gray-500">
                      Install a version manually or pick a verified build from the catalog.
                    </p>
                  </div>
                  <QuickStartGuide onOpenBrowse={() => setIsCatalogOpen(true)} />
                </div>
              ) : (
                <div className="space-y-3">
                  {filteredRuntimes.map((rt) => (
                    <RuntimeCard key={rt.id} runtime={rt} />
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Right Column: Health Diagnostics & Storage */}
          <div className="lg:col-span-5 space-y-6">
            <DoctorPanel />
            <DiskUsageBar />
          </div>
        </main>
      </div>

      <SettingsModal isOpen={isSettingsOpen} onClose={() => setIsSettingsOpen(false)} />
      <AvailableVersionsModal
        isOpen={isCatalogOpen}
        onClose={() => setIsCatalogOpen(false)}
        onInstalled={(t, v) => addToast(`Installed ${t} v${v}`, 'success')}
      />
      <UsageHistoryModal isOpen={isHistoryOpen} onClose={() => setIsHistoryOpen(false)} />
      <ToastContainer toasts={toasts} onDismiss={removeToast} />
    </div>
  );
}

export default App;
