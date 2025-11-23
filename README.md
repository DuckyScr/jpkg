# jpkg – Modern Java Package Manager

![jpkg logo](https://github.com/DuckyScr/jpkg/blob/main/jpkg.png)

> **jpkg** is a fast, cross‑platform Java package manager written in Rust. It resolves Maven artifacts, caches them for offline use, verifies integrity with SHA‑256 checksums, and works on macOS, Linux, and Windows.

---

## ✨ Features

- **Cross‑platform** – works on macOS, Linux, and Windows (classpath handling, path separators).  
- **Checksum verification** – `jpkg.lock` stores SHA‑256 hashes; use `--frozen` to enforce reproducible builds.  
- **Offline mode** – `--offline` uses a local cache at `~/.jpkg/cache/` for air‑gapped environments.  
- **Cache commands** – `jpkg cache list|clean|size` to manage cached JARs.  
- **Rich CLI** – colorized output, progress bars, and helpful error logs.  
- **Zero‑runtime dependencies** – a single binary after `cargo build --release`.

---

## 📦 Installation

### macOS / Linux (one‑liner)

```bash
sh -c "$(curl -fsSL https://raw.githubusercontent.com/DuckyScr/jpkg/main/install.sh)"
```

The script downloads the latest release binary, installs it to `/usr/local/bin/jpkg`, and makes it executable.  It also creates the cache directory (`~/.jpkg/cache/`).

### Windows (PowerShell one‑liner)

```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force; 
iex ((New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/DuckyScr/jpkg/main/install.ps1'))
```

The PowerShell script fetches the Windows binary, places it in `$Env:ProgramFiles\jpkg\jpkg.exe`, and adds that folder to the user's `PATH` for the current session.

---

## 🚀 Quick Start

```bash
# Initialise a new project
jpkg init myapp

# Add a dependency (e.g., Guava)
jpkg add com.google.guava:guava:31.1-jre

# Install dependencies (download & cache)
jpkg install

# Build and run
jpkg build && jpkg run
```

### Offline usage

```bash
# After a normal install, you can work without network
jpkg install --offline
```

The command will only succeed if all required JARs are already present in the cache.

---

## 🗂️ Cache Management

```bash
# List cached artifacts
jpkg cache list

# Show total cache size
jpkg cache size

# Remove all cached files
jpkg cache clean
```

---

## 🤝 Contributing

Contributions are welcome! Please fork the repository, create a feature branch, and open a pull request. Ensure that all tests pass (`cargo test`) and that the code follows the existing style.

---

## 📄 License

`jpkg` is released under the **MIT License**. See the `LICENSE` file for details.

---

*Built with ❤️ by the jpkg community.*
