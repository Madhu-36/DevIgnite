export type SupportedTool = 'node' | 'python' | 'go' | 'rust';

export interface Runtime {
  id: number;
  tool_name: string;
  version: string;
  path: string;
  sha256_checksum: string;
  is_active: boolean;
  installed_at: string;
}

export interface DiagnosticReport {
  symlink_healthy: boolean;
  symlink_target: string | null;
  devignite_in_path: boolean;
  permissions_ok: boolean;
  error_details: string[];
  auto_fix_available: boolean;
  is_healthy: boolean;
}

export interface AutoFixResult {
  fixed_items: string[];
  remaining_errors: string[];
  success: boolean;
}

export interface PathStatus {
  devignite_in_path: boolean;
  path_entries: string[];
  potential_collisions: string[];
}

export interface DownloadProgress {
  tool_name: string;
  version: string;
  downloaded_bytes: number;
  total_bytes: number | null;
  percentage: number;
}

export interface ToastMessage {
  id: string;
  type: 'success' | 'error' | 'info';
  message: string;
}
