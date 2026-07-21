import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type Page = "dashboard" | "catalog" | "doctor" | "settings" | "history";

interface InstalledRuntime {
  id: string;
  language: string;
  version: string;
  install_path: string;
  symlink_path: string;
  installed_at: string;
  is_active: boolean;
  checksum_verified: boolean;
  binary_name: string;
}

interface SandboxStatus {
  home_dir: string;
  sandbox_root: string;
  bin_dir: string;
  runtimes_dir: string;
  total_runtimes: number;
  total_symlinks: number;
  path_injected: boolean;
  disk_usage_bytes: number;
}

interface DoctorIssue {
  id: string;
  category: string;
  severity: string;
  description: string;
  affected_path: string | null;
  fixable: boolean;
  fix_applied: boolean;
}

interface DoctorReport {
  timestamp: string;
  issues: DoctorIssue[];
  summary: {
    total_checks: number;
    passed: number;
    warnings: number;
    errors: number;
    fixed: number;
  };
}

interface PathStatus {
  in_user_path: boolean;
  bin_dir_exists: boolean;
  symlink_count: number;
  sandbox_path: string;
}

interface InstallProgress {
  language: string;
  version: string;
  stage: string;
  progress: number;
  message: string;
}

interface HistoryEntry {
  0: string; 1: string; 2: string; 3: string; 4: string | null; 5: string;
}

const LANGUAGES: Record<string, { name: string; icon: string; binary: string; versions: string[] }> = {
  python: { name: "Python", icon: "🐍", binary: "python", versions: ["3.13.0", "3.12.7", "3.12.0", "3.11.6", "3.10.13"] },
  node: { name: "Node.js", icon: "🟢", binary: "node", versions: ["22.11.0", "21.2.0", "20.10.0", "18.19.0"] },
  rust: { name: "Rust", icon: "🦀", binary: "rustc", versions: ["1.83.0", "1.82.0", "1.81.0"] },
  go: { name: "Go", icon: "🔵", binary: "go", versions: ["1.23.3", "1.22.8", "1.21.4"] },
  java: { name: "Java", icon: "☕", binary: "java", versions: ["23.0.1", "21.0.4", "17.0.12", "11.0.24"] },
  gcc: { name: "GCC/G++", icon: "⚙️", binary: "gcc", versions: ["14.2.0", "13.3.0", "12.4.0"] },
  ruby: { name: "Ruby", icon: "💎", binary: "ruby", versions: ["3.3.5", "3.2.6", "3.1.6"] },
  deno: { name: "Deno", icon: "🦕", binary: "deno", versions: ["2.1.0", "1.46.3", "1.45.5"] },
};

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(1) + " " + units[i];
}

function timeAgo(dateStr: string): string {
  const diff = Date.now() - new Date(dateStr).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}

function App() {
  const [page, setPage] = useState<Page>("dashboard");
  const [runtimeCounts, setRuntimeCounts] = useState<Record<string, number>>({});
  const [allRuntimes, setAllRuntimes] = useState<InstalledRuntime[]>([]);
  const [sandboxStatus, setSandboxStatus] = useState<SandboxStatus | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [counts, runtimes, status] = await Promise.all([
        invoke<Record<string, number>>("get_runtime_counts"),
        invoke<InstalledRuntime[]>("get_installed_runtimes"),
        invoke<SandboxStatus>("get_sandbox_status"),
      ]);
      setRuntimeCounts(counts);
      setAllRuntimes(runtimes);
      setSandboxStatus(status);
    } catch (e) {
      console.error("Failed to refresh:", e);
    }
  }, []);

  useEffect(() => {
    refresh();
    const unlisten = listen("runtimes-changed", () => refresh());
    return () => { unlisten.then((fn) => fn()); };
  }, [refresh]);

  const totalInstalled = Object.values(runtimeCounts).reduce((a, b) => a + b, 0);

  return (
    <div className="app-layout">
      <nav className="sidebar">
        <div className="sidebar-header">
          <h1>DevIgnite</h1>
          <p>v0.2.0 — Environment Manager</p>
        </div>
        <div className="nav-items">
          {([
            ["dashboard", "📊", "Dashboard"],
            ["catalog", "📦", "Catalog"],
            ["doctor", "🩺", "Doctor"],
            ["history", "📋", "History"],
            ["settings", "⚙️", "Settings"],
          ] as [Page, string, string][]).map(([id, icon, label]) => (
            <button key={id} className={`nav-item ${page === id ? "active" : ""}`} onClick={() => setPage(id)}>
              <span className="icon">{icon}</span>{label}
            </button>
          ))}
        </div>
        <div className="sidebar-footer">
          <div className="status-line">
            <span className={`status-dot ${sandboxStatus?.path_injected ? "ok" : "error"}`} />
            {sandboxStatus?.path_injected ? "PATH active" : "PATH not injected"}
          </div>
          <div className="status-line">
            <span className="status-dot ok" />
            {totalInstalled} runtime{totalInstalled !== 1 ? "s" : ""} · {sandboxStatus?.total_symlinks ?? 0} symlinks
          </div>
        </div>
      </nav>
      <main className="main-content">
        {page === "dashboard" && <DashboardPage runtimes={allRuntimes} counts={runtimeCounts} status={sandboxStatus} onRefresh={refresh} />}
        {page === "catalog" && <CatalogPage onRefresh={refresh} />}
        {page === "doctor" && <DoctorPage />}
        {page === "history" && <HistoryPage />}
        {page === "settings" && <SettingsPage status={sandboxStatus} />}
      </main>
    </div>
  );
}

function DashboardPage({ runtimes, counts, status, onRefresh }: {
  runtimes: InstalledRuntime[];
  counts: Record<string, number>;
  status: SandboxStatus | null;
  onRefresh: () => void;
}) {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [testingId, setTestingId] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<Record<string, boolean>>({});
  const [switching, setSwitching] = useState<string | null>(null);

  const handleTest = async (rt: InstalledRuntime) => {
    setTestingId(rt.id);
    try {
      const result = await invoke<{ all_passed: boolean }>("run_smoke_test", {
        language: rt.language, runtimeId: rt.id,
      });
      setTestResults((prev) => ({ ...prev, [rt.id]: result.all_passed }));
    } catch {
      setTestResults((prev) => ({ ...prev, [rt.id]: false }));
    }
    setTestingId(null);
  };

  const handleSwitch = async (rt: InstalledRuntime) => {
    setSwitching(rt.id);
    try {
      await invoke("switch_version", { language: rt.language, runtimeId: rt.id });
      onRefresh();
    } catch (e) {
      console.error("Switch failed:", e);
    }
    setSwitching(null);
  };

  const handleUninstall = async (rt: InstalledRuntime) => {
    if (!confirm(`Remove ${rt.language} ${rt.version}?`)) return;
    try {
      await invoke("uninstall_runtime", { language: rt.language, runtimeId: rt.id });
      onRefresh();
    } catch (e) {
      console.error("Uninstall failed:", e);
    }
  };

  const grouped: Record<string, InstalledRuntime[]> = {};
  for (const rt of runtimes) {
    if (!grouped[rt.language]) grouped[rt.language] = [];
    grouped[rt.language].push(rt);
  }

  return (
    <div>
      <div className="page-header">
        <h2>Dashboard</h2>
        <p>Overview of your sandboxed development environment</p>
      </div>

      <div className="stats-row">
        {Object.entries(LANGUAGES).map(([key, lang]) => (
          <div key={key} className={`stat-card ${counts[key] > 0 ? "active" : ""}`}>
            <span className="stat-icon">{lang.icon}</span>
            <span className="stat-value">{counts[key] || 0}</span>
            <span className="stat-label">{lang.name}</span>
          </div>
        ))}
      </div>

      {status && (
        <div className="card info-card">
          <div className="info-grid">
            <div className="info-item"><span className="info-label">Sandbox Root</span><code>{status.sandbox_root}</code></div>
            <div className="info-item"><span className="info-label">Bin Dir</span><code>{status.bin_dir}</code></div>
            <div className="info-item"><span className="info-label">Disk Usage</span><span>{formatBytes(status.disk_usage_bytes)}</span></div>
            <div className="info-item"><span className="info-label">Active Symlinks</span><span>{status.total_symlinks}</span></div>
          </div>
        </div>
      )}

      {Object.keys(grouped).length === 0 ? (
        <div className="empty-state">
          <div className="icon">📭</div>
          <p>No runtimes installed yet</p>
          <button className="btn-primary" onClick={() => (window as any).__gotoCatalog?.()}>Browse Catalog</button>
        </div>
      ) : (
        Object.entries(grouped).map(([lang, rts]) => {
          const meta = LANGUAGES[lang] || { name: lang, icon: "📦", binary: "" };
          return (
            <div key={lang} className="card">
              <div className="card-header">
                <h3>{meta.icon} {meta.name}</h3>
                <span className="badge badge-success">{rts.length} version{rts.length !== 1 ? "s" : ""}</span>
              </div>
              <div className="runtime-list">
                {rts.map((rt) => (
                  <div
                    key={rt.id}
                    className={`runtime-row ${rt.is_active ? "active" : ""} ${selectedId === rt.id ? "selected" : ""}`}
                    onClick={() => setSelectedId(selectedId === rt.id ? null : rt.id)}
                  >
                    <div className="runtime-row-main">
                      <div className="runtime-info">
                        <span className="runtime-version">{rt.version}</span>
                        {rt.is_active && <span className="badge badge-success">Active</span>}
                        {rt.checksum_verified && <span className="badge badge-ok">Verified</span>}
                        {testResults[rt.id] !== undefined && (
                          <span className={`badge ${testResults[rt.id] ? "badge-success" : "badge-error"}`}>
                            {testResults[rt.id] ? "Passed" : "Failed"}
                          </span>
                        )}
                      </div>
                      <span className="runtime-path">{rt.install_path}</span>
                    </div>
                    {selectedId === rt.id && (
                      <div className="runtime-actions">
                        {!rt.is_active && (
                          <button className="btn-primary" onClick={(e) => { e.stopPropagation(); handleSwitch(rt); }} disabled={switching === rt.id}>
                            {switching === rt.id ? "Switching..." : "Switch to this"}
                          </button>
                        )}
                        <button className="btn-secondary" onClick={(e) => { e.stopPropagation(); handleTest(rt); }} disabled={testingId === rt.id}>
                          {testingId === rt.id ? "Testing..." : "Run Tests"}
                        </button>
                        <button className="btn-danger" onClick={(e) => { e.stopPropagation(); handleUninstall(rt); }}>
                          Remove
                        </button>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          );
        })
      )}
    </div>
  );
}

function CatalogPage({ onRefresh }: { onRefresh: () => void }) {
  const [selectedLang, setSelectedLang] = useState("python");
  const [installing, setInstalling] = useState<string | null>(null);
  const [progress, setProgress] = useState<InstallProgress | null>(null);
  const [installedVersions, setInstalledVersions] = useState<Set<string>>(new Set());

  const lang = LANGUAGES[selectedLang];

  useEffect(() => {
    invoke<InstalledRuntime[]>("get_runtimes_by_language", { language: selectedLang })
      .then((rts) => setInstalledVersions(new Set(rts.map((r) => r.version))))
      .catch(() => setInstalledVersions(new Set()));
  }, [selectedLang]);

  useEffect(() => {
    const unlisten = listen<InstallProgress>("install-progress", (event) => {
      setProgress(event.payload);
      if (event.payload.stage === "complete" || event.payload.stage === "error") {
        setTimeout(() => {
          setInstalling(null);
          setProgress(null);
          onRefresh();
          invoke<InstalledRuntime[]>("get_runtimes_by_language", { language: selectedLang })
            .then((rts) => setInstalledVersions(new Set(rts.map((r) => r.version))));
        }, 1500);
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [selectedLang, onRefresh]);

  const handleInstall = async (version: string) => {
    const binary = lang.binary;
    setInstalling(`${selectedLang}-${version}`);
    try {
      await invoke("install_runtime", {
        language: selectedLang,
        version,
        downloadUrl: "",
        sha256: "",
        binaryName: binary,
      });
    } catch (e) {
      console.error("Install failed:", e);
      setInstalling(null);
      setProgress(null);
    }
  };

  return (
    <div>
      <div className="page-header">
        <h2>Runtime Catalog</h2>
        <p>Install new programming language runtimes into your sandbox</p>
      </div>

      <div className="catalog-lang-bar">
        {Object.entries(LANGUAGES).map(([key, l]) => (
          <button
            key={key}
            className={`catalog-lang-btn ${selectedLang === key ? "active" : ""}`}
            onClick={() => setSelectedLang(key)}
          >
            <span>{l.icon}</span>{l.name}
          </button>
        ))}
      </div>

      <div className="card">
        <div className="card-header">
          <h3>{lang.icon} {lang.name} Versions</h3>
          <span className="badge badge-success">{installedVersions.size} installed</span>
        </div>

        {progress && installing && (
          <div className="install-progress-card">
            <div className="install-progress-header">
              <span>Installing {progress.language} {progress.version}...</span>
              <span className="badge badge-success">{progress.progress}%</span>
            </div>
            <div className="progress-bar">
              <div className="progress-bar-fill" style={{ width: `${progress.progress}%` }} />
            </div>
            <span className="install-progress-msg">{progress.message}</span>
          </div>
        )}

        <div className="catalog-versions">
          {lang.versions.map((v) => {
            const isInstalled = installedVersions.has(v);
            const isInstalling = installing === `${selectedLang}-${v}`;
            return (
              <div key={v} className={`catalog-version-row ${isInstalled ? "installed" : ""}`}>
                <div className="catalog-version-info">
                  <span className="catalog-version-num">{lang.name} {v}</span>
                  {isInstalled && <span className="badge badge-success">Installed</span>}
                  {isInstalling && progress && (
                    <span className="badge badge-success">{progress.progress}%</span>
                  )}
                </div>
                <button
                  className={isInstalled ? "btn-secondary" : "btn-primary"}
                  disabled={isInstalled || isInstalling}
                  onClick={() => handleInstall(v)}
                >
                  {isInstalled ? "Installed" : isInstalling ? "Installing..." : "Install"}
                </button>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function DoctorPage() {
  const [report, setReport] = useState<DoctorReport | null>(null);
  const [running, setRunning] = useState(false);
  const [fixingId, setFixingId] = useState<string | null>(null);

  const runDoctor = async (autoFix: boolean) => {
    setRunning(true);
    try {
      const result = await invoke<DoctorReport>("run_doctor", { autoFix });
      setReport(result);
    } catch (e) {
      console.error("Doctor failed:", e);
    }
    setRunning(false);
  };

  const fixIssue = async (issueId: string) => {
    setFixingId(issueId);
    try {
      await invoke("fix_doctor_issue", { issueId });
      await runDoctor(true);
    } catch (e) {
      console.error("Fix failed:", e);
    }
    setFixingId(null);
  };

  return (
    <div>
      <div className="page-header">
        <h2>DevIgnite Doctor</h2>
        <p>System diagnostics and conflict remediation</p>
      </div>

      <div className="doctor-actions">
        <button className="btn-primary" onClick={() => runDoctor(true)} disabled={running}>
          {running ? "Running..." : "Run Diagnostic & Auto-Fix"}
        </button>
        <button className="btn-secondary" onClick={() => runDoctor(false)} disabled={running}>
          Scan Only (No Fixes)
        </button>
      </div>

      {report && (
        <>
          <div className="stats-row">
            <div className="stat-card ok"><span className="stat-value">{report.summary.passed}</span><span className="stat-label">Passed</span></div>
            <div className="stat-card warning"><span className="stat-value">{report.summary.warnings}</span><span className="stat-label">Warnings</span></div>
            <div className="stat-card error"><span className="stat-value">{report.summary.errors}</span><span className="stat-label">Errors</span></div>
            <div className="stat-card fixed"><span className="stat-value">{report.summary.fixed}</span><span className="stat-label">Fixed</span></div>
          </div>

          <div className="card">
            <div className="card-header">
              <h3>Diagnostic Results</h3>
              <span className="info-text">{report.issues.length} checks completed</span>
            </div>
            <div className="doctor-grid">
              {report.issues.map((issue) => (
                <div key={issue.id} className={`doctor-issue ${issue.severity}`}>
                  <span className={`status-dot ${issue.severity}`} />
                  <div className="doctor-issue-content">
                    <span className="doctor-issue-category">{issue.category}</span>
                    <span className="doctor-issue-desc">{issue.description}</span>
                  </div>
                  {issue.fixable && !issue.fix_applied && (
                    <button
                      className="btn-fix"
                      onClick={() => fixIssue(issue.id)}
                      disabled={fixingId === issue.id}
                    >
                      {fixingId === issue.id ? "Fixing..." : "Fix"}
                    </button>
                  )}
                  {issue.fix_applied && <span className="badge badge-success">Fixed</span>}
                </div>
              ))}
            </div>
          </div>
        </>
      )}
    </div>
  );
}

function HistoryPage() {
  const [history, setHistory] = useState<HistoryEntry[]>([]);

  useEffect(() => {
    invoke<HistoryEntry[]>("get_install_history", { limit: 100 })
      .then(setHistory)
      .catch(() => {});
  }, []);

  return (
    <div>
      <div className="page-header">
        <h2>Install History</h2>
        <p>Log of all runtime operations</p>
      </div>
      <div className="card">
        {history.length === 0 ? (
          <div className="empty-state"><p>No history yet</p></div>
        ) : (
          <div className="history-list">
            {history.map((entry, i) => (
              <div key={i} className={`history-row ${entry[3]}`}>
                <span className="history-action">{entry[2]}</span>
                <span className="history-lang">{entry[0]}</span>
                <span className="history-ver">{entry[1]}</span>
                <span className={`badge ${entry[3] === "success" ? "badge-success" : "badge-error"}`}>{entry[3]}</span>
                <span className="history-time">{timeAgo(entry[5])}</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function SettingsPage({ status }: { status: SandboxStatus | null }) {
  const [pathStatus, setPathStatus] = useState<PathStatus | null>(null);

  useEffect(() => {
    invoke<PathStatus>("get_path_status").then(setPathStatus).catch(() => {});
  }, []);

  return (
    <div>
      <div className="page-header">
        <h2>Settings</h2>
        <p>Configure DevIgnite sandbox behavior</p>
      </div>

      <div className="card">
        <h3 style={{ marginBottom: 16 }}>Sandbox Paths</h3>
        {status && (
          <div className="settings-grid">
            <div className="settings-row"><span className="settings-label">Home Directory</span><code>{status.home_dir}</code></div>
            <div className="settings-row"><span className="settings-label">Sandbox Root</span><code>{status.sandbox_root}</code></div>
            <div className="settings-row"><span className="settings-label">Binary Directory</span><code>{status.bin_dir}</code></div>
            <div className="settings-row"><span className="settings-label">Runtimes Directory</span><code>{status.runtimes_dir}</code></div>
          </div>
        )}
      </div>

      <div className="card">
        <h3 style={{ marginBottom: 16 }}>PATH Management</h3>
        <div className="status-indicator">
          <span className={`status-dot ${pathStatus?.in_user_path ? "ok" : "error"}`} />
          <span>Sandbox in system PATH: <strong>{pathStatus?.in_user_path ? "Yes" : "No"}</strong></span>
        </div>
        <div className="settings-actions">
          <button className="btn-primary" onClick={() => invoke("ensure_path_injected").then(() => invoke<PathStatus>("get_path_status").then(setPathStatus))}>
            Inject PATH
          </button>
          <button className="btn-danger" onClick={() => invoke("remove_path_injection").then(() => invoke<PathStatus>("get_path_status").then(setPathStatus))}>
            Remove from PATH
          </button>
        </div>
      </div>

      <div className="card">
        <h3 style={{ marginBottom: 16 }}>Cache</h3>
        <div className="status-indicator">
          <span className="status-dot ok" />
          <span>Download cache: <code>~/.devignite/cache</code></span>
        </div>
      </div>
    </div>
  );
}

export default App;
