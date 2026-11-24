# DevIgnite Security Policy

DevIgnite prioritizes binary integrity, path isolation, and safe execution environments. This document outlines our vulnerability disclosure process, threat model, and built-in security controls.

---

## 1. Reporting Security Vulnerabilities

If you discover a security vulnerability in DevIgnite, please report it via private disclosure rather than public issues:
- **Email**: `security@devignite.dev`
- **PGP Key**: Available upon request
- **Response Window**: Initial response within 48 hours; assessment within 5 business days.

---

## 2. Threat Model & Safeguards

### 2.1 Zip-Slip & Path Traversal Mitigation
When unpacking compressed archives (`.zip`, `.tar.gz`, `.7z`), malicious archives may contain entries crafted with relative paths (e.g. `../../../../Windows/System32/evil.dll`).

**DevIgnite Mitigation**:
All extraction routines normalize target file paths and explicitly ensure `destination.starts_with(&base_sandbox_dir)`. If any archive entry resolves outside the sandbox, the entire extraction is aborted immediately and an error is logged.

### 2.2 Binary Tampering & Integrity Verification
Every downloaded archive is hashed using `SHA-256` in real-time as bytes stream from the network adapter. If the computed digest fails to match the expected signature or checksum:
1. The downloaded file is immediately deleted from `~/.devignite/cache/`.
2. No symlinks are created.
3. The database state remains untouched.

### 2.3 Non-Elevated Execution Principle
DevIgnite operates entirely within user-space (`~/.devignite/`). It does not require Administrative or Root privileges:
- On Windows, directory junctions or user-level soft links are used instead of machine-wide symlinks.
- Environment variable adjustments are restricted to `HKEY_CURRENT_USER\Environment\Path` without touching machine-level `HKEY_LOCAL_MACHINE`.
- On Linux and macOS, shell initialization files in `$HOME` are safely patched without modifying `/etc/profile` or `/etc/environment`.
