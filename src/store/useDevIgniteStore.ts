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
  symlink_healthy: boolean;
  symlink_target: string | null;
  devignite_in_path: boolean;
  permissions_ok: boolean;
  error_details: string[];
  auto_fix_available: boolean;
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
    set({ isLoading: true, errorMessage: null });
    try {
      const runtimes: Runtime[] = await invoke('get_installed_runtimes');
      set({ runtimes });
    } catch (error) {
      console.error('Failed to fetch runtimes', error);
      set({ errorMessage: String(error) });
    } finally {
      set({ isLoading: false });
    }
  },

  installRuntime: async (name, version) => {
    set({ isLoading: true, errorMessage: null });
    try {
      await invoke('install_runtime', { name, version });
      get().fetchRuntimes();
    } catch (error) {
      console.error('Failed to install runtime', error);
      set({ errorMessage: String(error) });
    } finally {
      set({ isLoading: false });
    }
  },

  switchVersion: async (name, version) => {
    set({ isLoading: true, errorMessage: null });
    try {
      await invoke('switch_version', { name, version });
      get().fetchRuntimes();
    } catch (error) {
      console.error('Failed to switch version', error);
      set({ errorMessage: String(error) });
    } finally {
      set({ isLoading: false });
    }
  },

  uninstallRuntime: async (name, version) => {
    set({ isLoading: true, errorMessage: null });
    try {
      await invoke('uninstall_runtime', { name, version });
      get().fetchRuntimes();
    } catch (error) {
      console.error('Failed to uninstall runtime', error);
      set({ errorMessage: String(error) });
    } finally {
      set({ isLoading: false });
    }
  },

  runDiagnostics: async () => {
    set({ isLoading: true, errorMessage: null });
    try {
      const diagnostics: DiagnosticReport = await invoke('run_diagnostics_cmd');
      set({ diagnostics });
    } catch (error) {
      console.error('Failed to run diagnostics', error);
      set({ errorMessage: String(error) });
    } finally {
      set({ isLoading: false });
    }
  },

  clearCache: async () => {
    set({ isLoading: true, errorMessage: null });
    try {
      await invoke('clear_cache');
    } catch (error) {
      console.error('Failed to clear cache', error);
      set({ errorMessage: String(error) });
    } finally {
      set({ isLoading: false });
    }
  },
}));
