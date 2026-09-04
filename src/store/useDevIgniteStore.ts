import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface Runtime {
  id: number;
  tool_name: string;
  version: string;
  path: string;
  sha256_checksum: string;
  is_active: boolean;
  installed_at: string;
}

interface DiagnosticReport {
  path_collisions: string[];
  broken_symlinks: string[];
  permission_issues: string[];
  is_healthy: boolean;
}

interface DevIgniteState {
  runtimes: Runtime[];
  diagnostics: DiagnosticReport | null;
  isLoading: boolean;
  errorMessage: string | null;
  setErrorMessage: (msg: string | null) => void;
  fetchRuntimes: () => Promise<void>;
  installRuntime: (name: string, version: string) => Promise<void>;
  switchVersion: (name: string, version: string) => Promise<void>;
  uninstallRuntime: (name: string, version: string) => Promise<void>;
  runDiagnostics: () => Promise<void>;
  clearCache: () => Promise<void>;
}

export const useDevIgniteStore = create<DevIgniteState>((set, get) => ({
  runtimes: [],
  diagnostics: null,
  isLoading: false,
  errorMessage: null,
  
  setErrorMessage: (msg) => set({ errorMessage: msg }),

  fetchRuntimes: async () => {
    set({ isLoading: true });
    try {
      const runtimes = await invoke<Runtime[]>('get_installed_runtimes');
      set({ runtimes });
    } catch (error) {
      console.error('Failed to fetch runtimes', error);
    } finally {
      set({ isLoading: false });
    }
  },

  installRuntime: async (name, version) => {
    set({ isLoading: true });
    try {
      await invoke('install_runtime', { name, version });
      await get().fetchRuntimes();
    } catch (error) {
      console.error('Failed to install runtime', error);
    } finally {
      set({ isLoading: false });
    }
  },

  switchVersion: async (name, version) => {
    set({ isLoading: true });
    try {
      await invoke('switch_version', { name, version });
      await get().fetchRuntimes();
    } catch (error) {
      console.error('Failed to switch version', error);
    } finally {
      set({ isLoading: false });
    }
  },

  uninstallRuntime: async (name, version) => {
    set({ isLoading: true });
    try {
      await invoke('uninstall_runtime', { name, version });
      await get().fetchRuntimes();
    } catch (error) {
      console.error('Failed to uninstall runtime', error);
    } finally {
      set({ isLoading: false });
    }
  },

  runDiagnostics: async () => {
    set({ isLoading: true });
    try {
      const diagnostics = await invoke<DiagnosticReport>('run_diagnostics_cmd');
      set({ diagnostics });
    } catch (error) {
      console.error('Failed to run diagnostics', error);
    } finally {
      set({ isLoading: false });
    }
  },

  clearCache: async () => {
    set({ isLoading: true });
    try {
      await invoke('clear_cache');
    } catch (error) {
      console.error('Failed to clear cache', error);
    } finally {
      set({ isLoading: false });
    }
  },
}));
