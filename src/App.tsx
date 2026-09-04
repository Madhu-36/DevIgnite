import React, { useEffect } from 'react';
import { useDevIgniteStore } from './store/useDevIgniteStore';
import { RuntimeCard } from './components/RuntimeCard';
import { DoctorPanel } from './components/DoctorPanel';
import { Layers } from 'lucide-react';

function App() {
  const { runtimes, fetchRuntimes, isLoading, errorMessage, setErrorMessage } = useDevIgniteStore();

  useEffect(() => {
    fetchRuntimes();
  }, [fetchRuntimes]);

  return (
    <div className="min-h-screen bg-gray-900 text-gray-100 p-8 font-sans">
      <div className="max-w-4xl mx-auto space-y-8">
        <header className="flex items-center justify-between border-b border-gray-800 pb-6">
          <div className="flex items-center space-x-3">
            <Layers className="w-8 h-8 text-blue-500" />
            <h1 className="text-3xl font-bold tracking-tight">DevIgnite</h1>
          </div>
          <button
            onClick={() => useDevIgniteStore.getState().clearCache()}
            disabled={isLoading}
            className="px-3 py-1.5 border border-red-800 text-red-400 hover:bg-red-900/30 rounded-md text-sm transition-colors disabled:opacity-50"
          >
            Clear Cache
          </button>
        </header>

        {errorMessage && (
          <div className="bg-red-900/40 border border-red-800 text-red-200 p-4 rounded-md flex justify-between items-center">
            <span>{errorMessage}</span>
            <button onClick={() => setErrorMessage(null)} className="text-red-400 hover:text-red-300">
              ✕
            </button>
          </div>
        )}

        <main className="grid grid-cols-1 md:grid-cols-2 gap-8">
          <div className="space-y-6">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-xl font-semibold text-gray-300">Installed Runtimes</h2>
            </div>
            
            <div className="bg-gray-800 p-4 rounded-lg flex space-x-3 items-center shadow-sm border border-gray-700">
              <input 
                id="toolName" 
                type="text" 
                placeholder="Tool (e.g. node)" 
                disabled={isLoading}
                className="bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm text-white focus:outline-none focus:border-blue-500 w-1/3 disabled:opacity-50"
              />
              <input 
                id="toolVersion" 
                type="text" 
                placeholder="Version (e.g. 20.10.0)" 
                disabled={isLoading}
                className="bg-gray-900 border border-gray-600 rounded px-3 py-1.5 text-sm text-white focus:outline-none focus:border-blue-500 w-1/3 disabled:opacity-50"
              />
              <button 
                onClick={() => {
                  const name = (document.getElementById('toolName') as HTMLInputElement).value.toLowerCase();
                  const version = (document.getElementById('toolVersion') as HTMLInputElement).value;
                  if (name && version) {
                    useDevIgniteStore.getState().installRuntime(name, version);
                  }
                }}
                disabled={isLoading}
                className="px-4 py-1.5 bg-blue-600 hover:bg-blue-700 rounded-md text-sm transition-colors font-medium w-1/3 disabled:opacity-50 flex justify-center items-center"
              >
                {isLoading ? <Layers className="animate-spin w-4 h-4 mr-2" /> : null}
                {isLoading ? 'Installing...' : 'Install'}
              </button>
            </div>
            
            <div>
              {runtimes.length === 0 ? (
                <div className="bg-gray-800 rounded-lg p-6 text-center text-gray-400 border border-gray-700 border-dashed">
                  No runtimes installed yet.
                </div>
              ) : (
                <div className="space-y-3">
                  {runtimes.map((rt) => (
                    <RuntimeCard key={rt.id} runtime={rt} />
                  ))}
                </div>
              )}
            </div>
          </div>

          <div>
            <DoctorPanel />
          </div>
        </main>
      </div>
    </div>
  );
}

export default App;
