import { useState } from "react";

type Page = "runtimes" | "install" | "doctor" | "settings";

function App() {
  const [currentPage, setCurrentPage] = useState<Page>("runtimes");

  return (
    <div className="app-layout">
      <nav className="sidebar">
        <div className="sidebar-header">
          <h1>DevIgnite</h1>
          <p>Environment Manager</p>
        </div>
        <div className="nav-items">
          <button
            className={`nav-item ${currentPage === "runtimes" ? "active" : ""}`}
            onClick={() => setCurrentPage("runtimes")}
          >
            <span className="icon">📦</span>
            Runtimes
          </button>
          <button
            className={`nav-item ${currentPage === "install" ? "active" : ""}`}
            onClick={() => setCurrentPage("install")}
          >
            <span className="icon">⚡</span>
            Install New
          </button>
          <button
            className={`nav-item ${currentPage === "doctor" ? "active" : ""}`}
            onClick={() => setCurrentPage("doctor")}
          >
            <span className="icon">🩺</span>
            Doctor
          </button>
          <button
            className={`nav-item ${currentPage === "settings" ? "active" : ""}`}
            onClick={() => setCurrentPage("settings")}
          >
            <span className="icon">⚙️</span>
            Settings
          </button>
        </div>
        <div className="sidebar-footer">
          <div className="status-line">
            <span className="status-dot ok" />
            Sandbox active
          </div>
        </div>
      </nav>
      <main className="main-content">
        {currentPage === "runtimes" && <RuntimesPage />}
        {currentPage === "install" && <InstallPage />}
        {currentPage === "doctor" && <DoctorPage />}
        {currentPage === "settings" && <SettingsPage />}
      </main>
    </div>
  );
}

function RuntimesPage() {
  const [selectedLang, setSelectedLang] = useState<string | null>(null);

  const languages = [
    { name: "Python", icon: "🐍", versions: ["3.12.0", "3.11.6", "3.10.13"] },
    { name: "Node.js", icon: "🟢", versions: ["21.2.0", "20.10.0", "18.19.0"] },
    { name: "Rust", icon: "🦀", versions: ["1.74.0"] },
    { name: "Go", icon: "🔵", versions: ["1.21.4"] },
    { name: "Java", icon: "☕", versions: ["21.0.1"] },
    { name: "GCC", icon: "⚙️", versions: ["13.2.0"] },
  ];

  return (
    <div>
      <div className="page-header">
        <h2>Installed Runtimes</h2>
        <p>Manage your programming language environments</p>
      </div>
      <div className="runtime-grid">
        {languages.map((lang) => (
          <div
            key={lang.name}
            className="runtime-card"
            onClick={() =>
              setSelectedLang(selectedLang === lang.name ? null : lang.name)
            }
          >
            <div className="runtime-card-header">
              <span className="lang-icon">{lang.icon}</span>
              <span className="badge badge-success">Active</span>
            </div>
            <h4>{lang.name}</h4>
            <div className="version">
              {lang.versions.length} version{lang.versions.length !== 1 ? "s" : ""} installed
            </div>
            <div className="version">Active: {lang.versions[0]}</div>
            {selectedLang === lang.name && (
              <div className="actions">
                <button className="btn-primary">Switch</button>
                <button className="btn-secondary">Test</button>
                <button className="btn-danger">Remove</button>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

function InstallPage() {
  const [selectedLanguage, setSelectedLanguage] = useState("python");
  const [installing, setInstalling] = useState(false);
  const [progress, setProgress] = useState(0);

  const availableLanguages = [
    { id: "python", name: "Python", versions: ["3.12.1", "3.12.0", "3.11.7", "3.11.6", "3.10.13"] },
    { id: "node", name: "Node.js", versions: ["21.2.0", "21.1.0", "20.10.0", "18.19.0"] },
    { id: "rust", name: "Rust", versions: ["1.74.0", "1.73.0"] },
    { id: "go", name: "Go", versions: ["1.21.4", "1.21.3"] },
    { id: "java", name: "Java (OpenJDK)", versions: ["21.0.1", "17.0.9", "11.0.21"] },
    { id: "gcc", name: "GCC/G++", versions: ["13.2.0", "12.3.0"] },
  ];

  const currentLang = availableLanguages.find((l) => l.id === selectedLanguage);

  const handleInstall = () => {
    setInstalling(true);
    setProgress(0);
    const interval = setInterval(() => {
      setProgress((p) => {
        if (p >= 100) {
          clearInterval(interval);
          setInstalling(false);
          return 100;
        }
        return p + 5;
      });
    }, 200);
  };

  return (
    <div>
      <div className="page-header">
        <h2>Install Runtime</h2>
        <p>Add a new programming language to your sandbox</p>
      </div>
      <div className="card">
        <div className="field">
          <label>Language</label>
          <select
            value={selectedLanguage}
            onChange={(e) => setSelectedLanguage(e.target.value)}
          >
            {availableLanguages.map((lang) => (
              <option key={lang.id} value={lang.id}>
                {lang.name}
              </option>
            ))}
          </select>
        </div>
        {currentLang && (
          <div className="field">
            <label>Version</label>
            <select>
              {currentLang.versions.map((v) => (
                <option key={v} value={v}>
                  {v}
                </option>
              ))}
            </select>
          </div>
        )}
        {installing && (
          <div>
            <div className="progress-bar">
              <div className="progress-bar-fill" style={{ width: `${progress}%` }} />
            </div>
            <p style={{ fontSize: 12, color: "var(--text-secondary)" }}>
              {progress < 20
                ? "Downloading..."
                : progress < 85
                  ? "Extracting..."
                  : progress < 95
                    ? "Verifying checksum..."
                    : "Setting up symlinks..."}
            </p>
          </div>
        )}
        <div style={{ marginTop: 16 }}>
          <button className="btn-primary" onClick={handleInstall} disabled={installing}>
            {installing ? "Installing..." : "Install"}
          </button>
        </div>
      </div>
    </div>
  );
}

function DoctorPage() {
  const [issues] = useState([
    { category: "Sandbox Integrity", status: "ok", description: "All sandbox directories exist and are accessible" },
    { category: "Symlink Health", status: "ok", description: "12 symlinks verified, all targets valid" },
    { category: "Shadowed Binaries", status: "warning", description: "System Python at /usr/bin/python may shadow sandboxed version" },
    { category: "PATH State", status: "ok", description: "Sandbox bin directory is in system PATH" },
    { category: "Orphaned Temp", status: "ok", description: "No orphaned temp files found" },
    { category: "Disk Usage", status: "ok", description: "Total runtime disk usage: 423.18 MB" },
  ]);

  return (
    <div>
      <div className="page-header">
        <h2>DevIgnite Doctor</h2>
        <p>System diagnostics and conflict resolution</p>
      </div>
      <div style={{ marginBottom: 16 }}>
        <button className="btn-primary">Run Full Diagnostic</button>
      </div>
      <div className="card">
        <div className="card-header">
          <h3>Diagnostic Results</h3>
          <span className="badge badge-warning">1 Warning</span>
        </div>
        <div className="doctor-grid">
          {issues.map((issue) => (
            <div key={issue.category} className="doctor-issue">
              <span className={`status-dot ${issue.status}`} />
              <span className="category">{issue.category}</span>
              <span className="description">{issue.description}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function SettingsPage() {
  return (
    <div>
      <div className="page-header">
        <h2>Settings</h2>
        <p>Configure DevIgnite behavior</p>
      </div>
      <div className="card">
        <h3 style={{ marginBottom: 16 }}>Sandbox Configuration</h3>
        <div className="status-indicator">
          <span className="status-dot ok" />
          <span style={{ fontSize: 13 }}>
            Sandbox bin path: <code>~/.devignite/bin</code>
          </span>
        </div>
        <div className="status-indicator">
          <span className="status-dot ok" />
          <span style={{ fontSize: 13 }}>PATH injection: Active</span>
        </div>
      </div>
      <div className="card">
        <h3 style={{ marginBottom: 16 }}>Cache</h3>
        <div className="status-indicator">
          <span className="status-dot ok" />
          <span style={{ fontSize: 13 }}>Download cache: ~/.devignite/cache</span>
        </div>
        <button className="btn-secondary" style={{ marginTop: 8 }}>
          Clear Cache
        </button>
      </div>
    </div>
  );
}

export default App;
