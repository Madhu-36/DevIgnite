# Contributing to DevIgnite

Thank you for your interest in contributing to DevIgnite! Follow this guide to set up your local development environment and submit pull requests.

---

## Prerequisites

- **Rust toolchain** (1.75+ recommended): Install via [rustup.rs](https://rustup.rs/)
- **Node.js** (v18 or v20) and **npm**: For building the React dashboard
- **OS-specific dependencies**:
  - **Windows**: Visual Studio C++ Build Tools, WebView2 Runtime (bundled with Windows 10/11)
  - **Linux**: `libwebkit2gtk-4.1-dev`, `build-essential`, `curl`, `wget`, `file`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`
  - **macOS**: Xcode Command Line Tools

---

## Quick Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/Madhu-36/DevIgnite.git
   cd DevIgnite
   ```

2. **Install frontend dependencies**:
   ```bash
   npm install
   ```

3. **Run in development mode**:
   ```bash
   npm run tauri dev
   ```

4. **Run tests**:
   ```bash
   # Rust backend tests
   cargo test --manifest-path src-tauri/Cargo.toml

   # Frontend build verification
   npm run build
   ```

---

## Coding Standards

- **Rust**: Format with `cargo fmt`, ensure `cargo clippy --all-targets` passes with no warnings. Never use `unwrap()` or `panic!` in runtime paths—propagate errors with `crate::error::Result<T>`.
- **TypeScript**: Adhere to ESLint and Prettier rules. Use functional React components with explicit hook dependencies.
- **Git Commits**: Follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`).
