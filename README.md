<p align="center">
  <img src="./img/tomorrow.jpg" alt="HYPERSCOOP Logo" height="420" style="border-radius: 12px; box-shadow: 0 8px 24px rgba(0,0,0,0.15);"/>
</p>

<h1 align="center">✨ HYPERSCOOP (<code>hp</code>)</h1>

<p align="center">
  <b>⚡ The Next-Generation, Blazing-Fast & Beautiful Windows Package Manager written in Rust</b><br>
  <em>100x Faster than PowerShell Scoop • Homebrew-Grade Ergonomics • Declarative Environment Management</em>
</p>

<p align="center">
  <a href="https://github.com/Super1Windcloud/hyperscoop/releases">
    <img src="https://img.shields.io/github/v/release/Super1Windcloud/hyperscoop?color=7c3aed&label=Release&logo=github&style=flat-square" alt="Latest Release">
  </a>
  <a href="https://github.com/Super1Windcloud/hyperscoop/actions/workflows/release_publish.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/Super1Windcloud/hyperscoop/release_publish.yml?label=CI%2FCD&logo=githubactions&style=flat-square" alt="Build Status">
  </a>
  <img src="https://img.shields.io/badge/Language-Rust%202024-orange?logo=rust&style=flat-square" alt="Rust 2024">
  <img src="https://img.shields.io/badge/Platform-Windows%20%7C%20x64%20%7C%20ARM64%20%7C%20x86-0078d4?logo=windows&style=flat-square" alt="Platform">
  <img src="https://img.shields.io/badge/License-MIT%2FApache--2.0-emerald?style=flat-square" alt="License">
  <a href="./README.ch.md">
    <img src="https://img.shields.io/badge/中文文档-点击阅读-blue?style=flat-square&logo=readme" alt="Chinese Documentation">
  </a>
</p>

<p align="center">
  <a href="#-why-hyperscoop">Why hp</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-killer-features">Killer Features</a> •
  <a href="#-homebrew-alignment">Homebrew Comparison</a> •
  <a href="#-command-cheatsheet">Command Cheatsheet</a> •
  <a href="#-declarative-hpfile">Hpfile (Bundle)</a> •
  <a href="#-project-status">Project Status</a>
</p>

<hr style="border: 0; height: 2px; background: linear-gradient(to right, #7c3aed, #06b6d4, #10b981);" />

> [!TIP]
> **Tired of PowerShell's slow startup lag and Scoop's sluggish manifest parsing?**  
> **hyperscoop (`hp`)** rewrites the entire Scoop ecosystem in pure Rust. It delivers **sub-millisecond instant execution**, multi-threaded parallel downloads, declarative environment replication via `Hpfile`, and Homebrew-grade CLI ergonomics on Windows!

---

## ⚡ Why hyperscoop?

| Dimension | 🐢 Legacy Scoop (PowerShell) | 🚀 **hyperscoop (`hp`)** | 🍏 macOS Homebrew (`brew`) |
| :--- | :--- | :--- | :--- |
| **Engine / Runtime** | Interpreted PowerShell script (Slow startup) | **Pure Native Rust (Compiled binary, zero runtime overhead)** | Ruby + Bash scripts |
| **Manifest Search (30,000+ apps)** | ~2,800 ms (Spins terminal spinner) | **~15 ms (100x+ faster, instantaneous)** | ~350 ms |
| **Hash Verification** | Single-threaded sequential calculation | **Hardware-accelerated parallel streaming SHA-256** | SHA-256 streaming |
| **Download Engine** | Manual or fixed Aria2 parameters | **Self-adaptive bandwidth saturation (Auto thread/shard tuning)** | `curl` backend |
| **Safety & Reverse Deps** | Accidental breaks when removing shared deps | **`hp uses` reverse dependency safety guard** | `brew uses` guard |
| **Declarative Backup** | ❌ Not available natively | **`hp bundle` (`Hpfile`) one-click backup & restore** | `brew bundle` (`Brewfile`) |
| **Environment Diagnostics** | Basic checks | **`hp doctor` (Auto-detects broken junctions, invalid shims, PATH)** | `brew doctor` |
| **Preview Mode** | ❌ No `--dry-run` | **Native `-n / --dry-run` for install/uninstall/cleanup/autoremove** | Supported |
| **Internationalization** | English only | **Native English & Simplified Chinese (`i18n`) auto-adaptive** | English only |

---

## 🚀 Quick Start

Choose any of the following convenient installation methods:

### Method 1: Via Scoop (Recommended for existing Scoop users)
```powershell
scoop bucket add hp https://github.com/Super1Windcloud/hyperscoop_source_bucket.git
scoop install -u -s hp/hp
```

### Method 2: Via PowerShell One-Liner (Instant Install)
```powershell
irm -useb https://raw.githubusercontent.com/Super1Windcloud/hyperscoop/refs/heads/main/install.ps1 | iex
```

### Method 3: Via Cargo (Compile directly from GitHub)
```bash
cargo install --git https://github.com/super1windcloud/hyperscoop
```

### Method 4: Standalone Binary Download
Download the prebuilt `hp.exe` directly from [GitHub Releases](https://github.com/Super1Windcloud/hyperscoop/releases) and add it to your system `$env:PATH`.

---

## 💎 Killer Features

### 1. 🏎️ Sub-Millisecond Native Rust Performance
Zero startup latency. Hyperscoop eliminates all PowerShell overhead by managing buckets, manifests, git repositories, and registry shims with raw Rust multithreading and memory-mapped files.

### 2. 🍏 Seamless Homebrew Migration Experience
If you've used macOS Homebrew, your fingers already know `hp`:
* `hp install <app>` / `hp reinstall <app>` / `hp upgrade <app>`
* `hp outdated` (supports `-q` for clean pipelines and `--json` for scripts)
* `hp tap` (no args lists all taps, just like `brew tap`) & `hp untap <bucket>`
* `hp uses <app>` (reverse dependency inspection before uninstallation)
* `hp leaves` (list standalone top-level apps that aren't dependencies)
* `hp doctor` (thorough diagnostics of shims, junctions, and PATH order)

### 3. 📦 Declarative Environment Management (`Hpfile`)
Migrating to a brand new Windows workstation? Recreate your entire development toolchain with a single command:
```powershell
# Export your current tools, buckets, and configurations into an Hpfile
hp bundle dump

# On your new PC: Reinstall and configure everything in parallel!
hp bundle install
```

### 4. ⚡ Adaptive Bandwidth Maximizer
No need to tinker with complicated Aria2 parameters. Hyperscoop dynamically analyzes package sizes and adjusts connections, chunk sizes, and concurrent shards to completely saturate your broadband connection.

### 5. 🛡️ Reverse Dependency Protection (`hp uses`)
Never accidentally brick your toolchains again. When removing an app, Hyperscoop verifies whether other installed tools depend on it, warning you with clear dependency trees.

---

## 🧰 Command Cheatsheet

```text
hp [COMMAND] [OPTIONS]
```

### 📦 Package Operations
| Command | Alias | Description |
| :--- | :--- | :--- |
| `hp install <app>` | `hp i` | Install application(s). Supports `-n / --dry-run`, `-f / --force`, `--no-deps` |
| `hp reinstall <app>` | `hp re` | Reinstall application(s) cleanly |
| `hp uninstall <app>` | `hp rm`, `un` | Uninstall application with reverse-dependency safety check |
| `hp update [app]` | `hp upgrade` | Update packages; pass `-c / --cleanup` to auto-delete older versions |
| `hp outdated` | `hp status` | List apps with pending updates. Supports `-q / --quiet` and `--json` |
| `hp fetch <app>` | `hp download` | Download installer archive into cache and verify hash without installing |

### 🔍 Search & Inspection
| Command | Alias | Description |
| :--- | :--- | :--- |
| `hp search <query>` | `hp s` | Blazing-fast parallel fuzzy & exact manifest search across all buckets |
| `hp info <app>` | `hp desc` | Show detailed package manifest metadata, license, and post-install caveats |
| `hp cat <app>` | — | Print colorized manifest JSON definition in terminal |
| `hp edit <app>` | — | Open the local manifest in your default editor (`$EDITOR` / VS Code / Notepad) |
| `hp which <command>` | — | Identify which Scoop package provided a specific CLI executable |
| `hp prefix <app>` | — | Print the installed folder path prefix of an application |

### 🌲 Dependency & Topology Management
| Command | Description |
| :--- | :--- |
| `hp deps <app>` | Inspect package dependencies (supports `--tree` and `-r / --reverse`) |
| `hp uses <app>` | Query which applications depend on the target package (supports `-q` and `--installed`) |
| `hp leaves` | List standalone top-level apps (packages not required by any other app) |
| `hp missing` | Scan your system and report missing runtime dependencies |
| `hp autoremove` | Clean up orphaned dependencies no longer needed by any app |

### 🔫 Taps & Bucket Management
| Command | Alias | Description |
| :--- | :--- | :--- |
| `hp tap` | `hp bucket` | List all subscribed bucket sources (URL, last updated, manifest count) |
| `hp tap <name> [url]` | `hp bucket add` | Add an official or custom git bucket |
| `hp untap <name>` | `hp bucket rm` | Remove a bucket repository |
| `hp bucket known` | `hp b k` | List all well-known community and official buckets |

### ⚙️ System & Environment
| Command | Description |
| :--- | :--- |
| `hp doctor` / `checkup` | Diagnose Windows PATH ordering, broken symlinks, orphaned shims, and antivirus conflicts |
| `hp cleanup` | Reclaim disk space by purging old versions and download caches (`-n` for preview) |
| `hp size` / `disk-usage` | Breakdown disk space consumed by apps, cache, and persistent directories |
| `hp hold <app>` / `hp pin` | Lock an app's version to prevent unintended updates (`hp unpin` to unlock) |
| `hp switch <app> [ver]` | Switch active version of a multi-version application |
| `hp services` / `service` | Manage Windows background services registered by packages (start/stop/restart) |
| `hp shellenv` | Generate shell environment evaluation exports for PowerShell, Bash, Zsh, Nushell |
| `hp bundle` | Manage declarative `Hpfile` workflows (`dump`, `install`, `check`, `cleanup`) |

---

## 📋 Declarative `Hpfile` Showcase

Managing tools with an `Hpfile` gives you reproducible, version-controlled developer environments across any number of Windows machines:

```ruby
# Example Hpfile
bucket "extras"
bucket "nerd-fonts"
bucket "versions"

app "git"
app "neovim"
app "ripgrep"
app "fzf"
app "rust-analyzer"
app "JetBrainsMono-NF"
```

```powershell
# Check for any drift between installed packages and your Hpfile
hp bundle check

# Install all missing buckets and applications in parallel
hp bundle install
```

---

## 🎨 Terminal Preview

<p align="center">
  <img src="./img/en_cli.png" width="760" alt="Hyperscoop CLI Preview" style="border-radius: 8px;"/>
</p>

---

## 📦 Bucket Ecosystem

Hyperscoop is 100% compatible with the official Scoop bucket ecosystem, community buckets, and custom enterprise git repositories:

<p align="center">
  <img src="./img/bucket.png" width="720" alt="Bucket Demo" style="border-radius: 8px;"/>
</p>

---

## 🌟 Philosophy

> *“Beauty is a choice — a form of restraint, not greed.”*  
> *“A gentleman should be flexible like wood and wild geese, adapting with the transformations of dragons and serpents.”*

<p align="center">
  <img src="./img/flowser.jpg" width="360" alt="Flower illustration" style="border-radius: 8px; margin-right: 12px;"/>
  <img src="./img/sea.jpg" width="360" alt="Sea illustration" style="border-radius: 8px;"/>
</p>

---

## 🤝 Contributing & Community

Contributions, issues, and feature requests are warmly welcomed!  
Feel free to open an [issue](https://github.com/Super1Windcloud/hyperscoop/issues) or submit a [pull request](https://github.com/Super1Windcloud/hyperscoop/pulls).

If you find **hyperscoop** helpful, please give it a ⭐️ **Star** on GitHub to help more developers discover it!

<p align="center">
  <a href="https://github.com/Super1Windcloud/hyperscoop">
    <img src="https://api.star-history.com/svg?repos=Super1Windcloud/hyperscoop&type=Date" alt="Star History Chart" width="600"/>
  </a>
</p>

<p align="center">
  Copyright © 2026 <a href="https://github.com/Super1Windcloud">superwindcloud</a>. Released under the MIT or Apache-2.0 License.
</p>
