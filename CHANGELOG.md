# Changelog

All notable changes to DevIgnite will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.3.0] - 2026-09-04

### Added
- Implemented `get_system_config` and `set_system_config` Tauri backend commands for SQLite configuration management.
- Implemented `clear_cache` command to flush `~/.devignite/cache/` on demand.
- Added global error reporting state and dismissible alert banner in React frontend.
- Added manual runtime list refresh trigger to synchronization view.
- Added Auto-Fix metadata flags to DevIgnite Doctor diagnostics.

---

## [0.2.0] - 2026-08-22

### Added
- Multi-runtime support for Go, Python standalone binaries, and Rust toolchains.
- Dynamic runtime installer text inputs for arbitrary semver releases.
- Windows PATH registry injection via `winreg` writing to `HKCU\Environment\Path`.
- Broadcast of `WM_SETTINGCHANGE` across `HWND_BROADCAST` without requiring system reboot.
- Switch version and uninstall runtime IPC commands.

---

## [0.1.0] - 2026-08-20

### Added
- Initial project scaffolding with Tauri v2 + React + TypeScript + Tailwind CSS.
- Core Sandboxed Symlink Engine for Windows NTFS Junctions and POSIX links.
- Streaming HTTP download pipeline with SHA-256 integrity verification.
- Zip-Slip path-traversal prevention for `.zip` and `.tar.gz` extraction.
- Embedded SQLite state database tracking installed runtimes and active symlinks.
- DevIgnite Doctor diagnostics panel for detecting PATH collisions and orphaned links.
