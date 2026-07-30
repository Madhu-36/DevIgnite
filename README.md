DevIgnite
DevIgnite is a high-performance, cross-platform developer environment manager that lets you install, manage, and switch between programming language runtimes with a single click. Designed to eliminate "path pollution" and installation fatigue, DevIgnite keeps your system clean by using a sandboxed symlink routing engine.
🚀 Key Features
1.⚡ Zero-Config Installs: Install Python, Node.js, Rust, Go, and more without manual setup.
.
2. 🛡️ Sandboxed Routing: Keeps your OS pristine by routing all tools through a single, controlled binary directory—no more cluttered System PATH.

3. 🔄 Instant Switching: Switch between different versions (e.g., Python 3.10 to 3.12) instantly without reinstalling.

4. 🔒 Cryptographically Secure: Automatically verifies SHA-256 checksums of all downloaded binaries to ensure security.

🩺 DevIgnite Doctor: Built-in diagnostics to detect and resolve environment conflicts or broken dependencies.

🚀 Lightweight Performance: Built with Rust and Tauri for a fast, memory-efficient experience.

🏗️ How It Works (The Sandbox Architecture)
Unlike standard installers that dump files across your OS, DevIgnite employs a central sandbox approach:

Isolated Runtimes: All language binaries are stored in a dedicated, versioned directory structure (~/.devignite/runtimes/).

Symlink Orchestration: A single ~/.devignite/bin/ directory is added to your PATH. DevIgnite dynamically updates symlinks within this folder to point to your active compiler versions.

OS-Native Integration: On Windows, DevIgnite uses the Windows API to broadcast environment changes (WM_SETTINGCHANGE), allowing your terminal and IDEs to recognize new installations instantly—no reboot required.

🛠️ Tech Stack
Core Engine: Rust (High-performance OS-level orchestration)

Frontend UI: Tauri + React (Modern, lightweight desktop interface)

Data Layer: SQLite (Persistent state management)

📦 Getting Started (Planned)
(You can add build instructions here later as you progress!)
Bash
# Clone the repository.
git clone https://github.com/yourusername/DevIgnite.git
# Install dependenciesnpm install
# Run the development build
npm run tauri dev
📈 Roadmap
[ ] MVP: Core CLI engine for Python & Node.js installation.
[ ] UI Integration: Full Tauri dashboard for language management.

[ ] Doctor Module: Automated environment conflict resolution.

[ ] Config Files: devignite.toml support for project-specific environment auto-switching.

🤝 Contributing
Contributions are welcome! Whether it's adding support for a new language, improving the Rust orchestration engine, or enhancing the UI, feel free to open a Pull Request.
