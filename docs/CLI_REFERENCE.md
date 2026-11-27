# DevIgnite CLI Reference

DevIgnite supports dual execution modes: an intuitive desktop UI powered by Tauri v2 and a lightweight headless command-line interface for terminal scripting and continuous integration workflows.

---

## 1. Subcommands

### `devignite list` (or `devignite ls`)
Displays all currently installed runtimes and highlights the active symlink targets in `~/.devignite/bin/`.

```bash
$ devignite list
Active Runtimes:
  * node    v20.11.0  (~/.devignite/runtimes/node/20.11.0)
    node    v18.19.0  (~/.devignite/runtimes/node/18.19.0)
  * python  v3.12.1   (~/.devignite/runtimes/python/3.12.1)
  * go      v1.22.0   (~/.devignite/runtimes/go/1.22.0)
```

### `devignite use <tool> <version>`
Switches the active symlink for the specified tool instantly without touching the machine-wide System PATH or requiring a terminal reboot.

```bash
$ devignite use node 18.19.0
[OK] Switched node symlink to v18.19.0 (target: ~/.devignite/runtimes/node/18.19.0)
```

### `devignite doctor`
Runs comprehensive diagnostic checks on symlinks, file permissions, and PATH shadowing.

```bash
$ devignite doctor
[OK] Symlinks intact in ~/.devignite/bin
[OK] DevIgnite is registered in User PATH
[OK] Write permissions confirmed across ~/.devignite/
System health: 100% (Healthy)
```

---

## 2. Environment Variables

| Variable | Default Value | Description |
| :--- | :--- | :--- |
| `DEVIGNITE_ROOT` | `~/.devignite` | Custom base directory for runtime storage, bin directory, and cache |
| `DEVIGNITE_LOG` | `info` | Logging verbosity (`error`, `warn`, `info`, `debug`, `trace`) |
