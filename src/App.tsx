import React, { useEffect } from 'react';
import { useDevIgniteStore } from './store/useDevIgniteStore';
import { RuntimeCard } from './components/RuntimeCard';
import { DoctorPanel } from './components/DoctorPanel';
import { Layers } from 'lucide-react';

function App() {
  const { runtimes, fetchRuntimes } = useDevIgniteStore();

  useEffect(() => {
    fetchRuntimes();
  }, [fetchRuntimes]);

  return (
    <div className="min-h-screen bg-gray-900 text-gray-100 p-8 font-sans">
      <div className="max-w-4xl mx-auto space-y-8">
        <header className="flex items-center space-x-3 border-b border-gray-800 pb-6">
          <Layers className="w-8 h-8 text-blue-500" />
          <h1 className="text-3xl font-bold tracking-tight">DevIgnite</h1>
        </header>

        <main className="grid grid-cols-1 md:grid-cols-2 gap-8">
          <div className="space-y-6">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-xl font-semibold text-gray-300">Installed Runtimes</h2>
              <button 
                onClick={() => useDevIgniteStore.getState().installRuntime("node", "20.10.0")}
                className="px-3 py-1 bg-green-600 hover:bg-green-700 rounded-md text-sm transition-colors"
              >
                Install Node 20.10.0
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
