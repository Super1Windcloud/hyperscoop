<p align="center">
  <img src="./img/tomorrow.jpg" alt="HYPERSCOOP Logo" height="420" style="border-radius: 12px; box-shadow: 0 8px 24px rgba(0,0,0,0.15);"/>
</p>

<h1 align="center">✨ HYPERSCOOP (<code>hp</code>)</h1>

<p align="center">
  <b>⚡ 次世代更快、更强、更精美的 Windows 原生包管理器（Rust 纯血打造）</b><br>
  <em>继承 Scoop 庞大生态 • 告别 PowerShell 缓慢卡顿 • 带来 Homebrew 级的丝滑体验</em>
</p>

<p align="center">
  <a href="https://github.com/Super1Windcloud/hyperscoop/releases">
    <img src="https://img.shields.io/github/v/release/Super1Windcloud/hyperscoop?color=7c3aed&label=%E6%9C%80%E6%96%B0%E7%89%88%E6%9C%AC&logo=github&style=flat-square" alt="最新版本">
  </a>
  <a href="https://github.com/Super1Windcloud/hyperscoop/actions/workflows/release_publish.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/Super1Windcloud/hyperscoop/release_publish.yml?label=CI%2FCD%20%E6%B5%81%E6%B0%B4%E7%BA%BF&logo=githubactions&style=flat-square" alt="构建状态">
  </a>
  <img src="https://img.shields.io/badge/%E5%BC%80%E5%8F%91%E8%AF%AD%E8%A8%80-Rust%202024-orange?logo=rust&style=flat-square" alt="Rust 2024">
  <img src="https://img.shields.io/badge/%E6%94%AF%E6%8C%81%E5%B9%B3%E5%8F%B0-Windows%20x64%20%7C%20ARM64%20%7C%20x86-0078d4?logo=windows&style=flat-square" alt="Windows 平台">
  <img src="https://img.shields.io/badge/%E5%BC%80%E6%BA%90%E5%8D%8F%E8%AE%AE-MIT%2FApache--2.0-emerald?style=flat-square" alt="开源协议">
  <a href="./README.md">
    <img src="https://img.shields.io/badge/English-Documentation-blue?style=flat-square&logo=readme" alt="English Documentation">
  </a>
</p>

<p align="center">
  <a href="#-为什么选择-hyperscoop">为什么选择 hp</a> •
  <a href="#-快速开始">快速开始</a> •
  <a href="#-杀手级核心特性">核心特性</a> •
  <a href="#-对比传统-scoop--mac-homebrew">横向对比</a> •
  <a href="#-全功能命令速查表">命令速查</a> •
  <a href="#-声明式环境管理-hpfile">声明式 Hpfile</a> •
  <a href="#-项目理念">项目理念</a>
</p>

<hr style="border: 0; height: 2px; background: linear-gradient(to right, #7c3aed, #06b6d4, #10b981);" />

> [!TIP]
> **还在忍受原生 Scoop 每次启动 PowerShell 时的漫长转圈和卡顿？**  
> **hyperscoop (`hp`)** 用原生 Rust 对底层架构进行了颠覆式重构，拥有 **毫秒级极速响应**、智能多线程并发下载、基于 `Hpfile` 的声明式一键配机，以及与 macOS Homebrew 高度对齐的现代命令行体验！

---

## ⚡ 为什么选择 hyperscoop？

| 核心维度 | 🐢 传统原生 Scoop (PowerShell) | 🚀 **hyperscoop (`hp`)** | 🍏 macOS Homebrew (`brew`) |
| :--- | :--- | :--- | :--- |
| **底层引擎 / 运行时** | 解释型 PowerShell 脚本（冷启动极慢） | **纯 Rust 原生编译二进制（零依赖，微秒级启动）** | Ruby + 各种系统脚本 |
| **海量清单搜索 (30,000+ 应用)** | ~2,800 ms（明显停顿卡顿） | **~15 ms（提升 100 倍以上，敲下回车即出）** | ~350 ms |
| **安装包哈希校验** | 单线程串行计算，大文件易卡住 | **硬件级指令集加速、多线程流式校验** | SHA-256 流式校验 |
| **下载并发加速** | 需繁琐手动配置 Aria2 参数 | **自适应网络带宽最大化（动态优化分片与线程）** | 系统 curl 下载 |
| **反向依赖安全保护** | 卸载基础包极易连带弄坏其他软件 | **内置 `hp uses` 反向依赖智能防护与警示** | `brew uses` 防护 |
| **声明式环境装机** | ❌ 官方无开箱即用方案 | **`hp bundle` (`Hpfile`) 一键导出与秒级还原** | `brew bundle` (`Brewfile`) |
| **系统体检与修复** | 依赖简单诊断 | **`hp doctor` / `checkup -f`（自动定位并支持一键就地修复断开软链、无效 Shim、长路径与 PATH）** | `brew doctor` |
| **流程预演支持** | ❌ 不支持无损预览 | **原生全流程 `-n / --dry-run` 预演** | 广泛支持 |
| **多语言本地化** | 仅支持英文 | **原生中英双语国际化 (`i18n`)，开箱即用** | 仅英文 |

---

## 🚀 快速开始

提供四种便捷安装方式，任选其一即可：

### 方式一：已有 Scoop 环境直接安装（推荐老用户）
```powershell
scoop bucket add hp https://github.com/Super1Windcloud/hyperscoop_source_bucket.git
scoop install -u -s hp/hp
```

### 方式二：PowerShell 一键免配置安装（推荐新机，开箱即用）
```powershell
# 标准安装（全自动配置环境变量、初始化官方 main 桶与 PowerShell 自动补全）：
irm -useb https://raw.githubusercontent.com/Super1Windcloud/hyperscoop/refs/heads/main/install.ps1 | iex

# 中国大陆网络镜像加速安装（免除 GitHub 连通性困扰）：
& ([scriptblock]::Create((irm -useb https://raw.githubusercontent.com/Super1Windcloud/hyperscoop/refs/heads/main/install.ps1))) -China
```

### 方式三：通过 Cargo 源码安装（Rust 开发者）
```bash
cargo install --git https://github.com/super1windcloud/hyperscoop
```

### 方式四：手动下载独立可执行文件
前往 [GitHub Releases](https://github.com/Super1Windcloud/hyperscoop/releases) 下载适合您架构的 `hp.exe`，直接解压并加入系统的 `$env:PATH` 即可使用。

---

## 💎 杀手级核心特性

### 1. 🏎️ 纯 Rust 纯血驱动，毫秒级响应
完全脱离 PowerShell 缓慢的运行时环境，用 Rust 并发原语与内存映射重写了 Bucket、Manifest 解析、Git 同步和 Shim 生成，体验从“等几秒”变为“瞬间完成”。

### 2. 🍏 对齐 macOS Homebrew 的现代命令行体验
如果你习惯了 macOS 下的 `brew`，你的肌肉记忆无需改变：
* `hp install <app>` / `hp reinstall <app>` / `hp upgrade <app>`（首次运行若无软件源，自动触发冷启动自愈拉取 `main` 桶）
* `hp outdated`（支持 `-q` 管道输出与 `--json` 格式化导出）
* `hp tap`（无参数直接列出已订阅仓库，与 `brew tap` 完全一致）与 `hp tap <name>` / `hp untap <bucket>`（知名官方桶如 `extras`、`versions` 无需填写 URL，直接添加）
* `hp uses <app>`（卸载基础软件前反查谁依赖了它）
* `hp leaves`（列出所有顶层软件，排查无用孤儿软件）
* `hp doctor` / `hp doctor --fix`（全系统环境体检，并支持 `-f / --fix` 一键就地自动修复）

### 3. 📦 声明式环境一键备份与迁移（`Hpfile`）
新买了一台 Windows 电脑，或者需要为同事配置一模一样的开发环境？
```powershell
# 1. 在老机器上一键导出所有软件与配置到 Hpfile
hp bundle dump

# 2. 在新机器上运行，全自动并行下载安装整套开发工具链！
hp bundle install
```

### 4. ⚡ 智能带宽拉满（Aria2 动态自适应）
内置自适应多线程算法，根据目标文件体积与网络条件动态调整分片大小和连接数，彻底压榨千兆宽带潜能。

### 5. 🛡️ 反向依赖安全防护 (`hp uses`)
在准备卸载 Python、Git 或某个底层库时，运行 `hp uses python` 即可查看当前电脑上还有哪些已安装软件依赖于它，避免“卸载一时爽，全局火葬场”。

---

## 🧰 全功能命令速查表

```text
hp [COMMAND] [OPTIONS]
```

### 📦 软件安装与生命周期
| 命令 | 别名 | 功能说明 |
| :--- | :--- | :--- |
| `hp install <app>` | `hp i` | 安装应用。支持 `-n / --dry-run` 预演、`-f / --force` 覆盖、`--no-deps` 跳过依赖（空库自愈） |
| `hp reinstall <app>` | `hp re` | 重新安装已有应用（保留配置与数据） |
| `hp uninstall <app>` | `hp rm`, `un` | 卸载应用（内置反向依赖安全保护） |
| `hp update [app]` | `hp upgrade` | 升级软件；支持 `-c / --cleanup` 自动清理升级前的历史遗留版本 |
| `hp outdated` | `hp status` | 查看有待更新的软件。支持 `-q / --quiet`（仅输出包名）与 `--json` |
| `hp fetch <app>` | `hp download` | 仅下载软件包到本地缓存并校验 SHA256 哈希，不触发安装 |

### 🔍 搜索与信息探查
| 命令 | 别名 | 功能说明 |
| :--- | :--- | :--- |
| `hp search <query>` | `hp s` | 跨所有 Bucket 毫秒级多线程模糊 / 精准搜索（空库自愈） |
| `hp info <app>` | `hp desc` | 查看软件详细元数据、许可证、安装说明与 Caveats 提示 |
| `hp cat <app>` | — | 在终端中高亮显示对应软件的 Manifest JSON 原始内容 |
| `hp edit <app>` | — | 在默认编辑器（VS Code / Notepad / `$EDITOR`）中直接打开清单文件 |
| `hp which <cmd>` | — | 快速定位某个终端命令是由哪个 Scoop 软件包提供的 |
| `hp prefix <app>` | — | 打印指定软件在本地的绝对安装根路径 |

### 🌲 依赖与拓扑关系
| 命令 | 功能说明 |
| :--- | :--- |
| `hp deps <app>` | 查看指定应用的依赖列表，支持 `--tree`（树状图）与 `-r / --reverse`（反查） |
| `hp uses <app>` | 反向查询当前有哪些已安装应用依赖了指定软件（支持 `-q` 管道传参） |
| `hp leaves` | 查看所有叶子节点应用（即不作为任何其他应用依赖的顶层应用） |
| `hp missing` | 扫描并报告系统中依赖破损或缺失的应用 |
| `hp autoremove` | 一键清理曾经作为依赖被安装、现已无人使用的孤儿包 |

### 🔫 软件源与 Bucket 管理
| 命令 | 别名 | 功能说明 |
| :--- | :--- | :--- |
| `hp tap` | `hp bucket` | 无参数时直接列出所有已添加的 Bucket（名称、URL、更新时间、应用数） |
| `hp tap <name> [url]` | `hp bucket add` | 添加官方或第三方 Git 清单仓库（已知官方桶如 `main`、`extras` 免输 URL） |
| `hp untap <name>` | `hp bucket rm` | 移除指定的 Bucket 仓库 |
| `hp bucket known` | `hp b k` | 列出社区与官方已知的热门 Bucket 清单 |

### ⚙️ 环境体检与系统维护
| 命令 | 功能说明 |
| :--- | :--- |
| `hp doctor` / `checkup` | 全面体检与自愈：检测断开软链、无效 Shim、PATH 环境变量与长路径支持，加 `-f / --fix` 一键自动修复 |
| `hp cleanup` | 清理旧版本软件与安装包缓存，释放宝贵硬盘空间（支持 `-n` 预演） |
| `hp size` / `disk-usage` | 可视化分析各应用、缓存和持久化数据的磁盘占用排行 |
| `hp hold <app>` / `hp pin` | 锁定应用版本，禁止其被更新（解锁命令：`hp unpin` / `hp unhold`） |
| `hp switch <app> [ver]` | 多版本共存管理：在已安装的多个历史版本之间无缝切换活动版本 |
| `hp services` / `service` | 管理软件注册的 Windows 后台服务（启动 / 停止 / 重启 / 状态查看） |
| `hp shellenv` | 导出当前环境 PATH 配置，支持 PowerShell, CMD, Bash, Zsh, Nushell |
| `hp bundle` | 声明式环境管理（`dump` 导出、`install` 一键装机、`check` 对齐检测） |

---

## 📋 声明式 `Hpfile` 实战

通过 `Hpfile` 将所有生产力工具代码化管理，纳入 Git 版本控制：

```ruby
# 生产力环境 Hpfile 示例
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
# 检查当前电脑软件与 Hpfile 的差异
hp bundle check

# 一键多线程并行还原全部工具与配置
hp bundle install
```

---

## 🎨 终端美学预览

<p align="center">
  <img src="./img/cmd1.png" width="760" alt="Hyperscoop 中文终端预览" style="border-radius: 8px;"/>
</p>

---

## 📦 丰富的 Bucket 生态

完美无缝兼容 Scoop 官方以及社区成千上万个优质 Bucket：

<p align="center">
  <img src="./img/bucket.png" width="720" alt="Bucket 生态预览" style="border-radius: 8px;"/>
</p>

---

## 🌟 项目哲学

> “美是一种选择，甚至是一种放弃，而不是贪婪。”  
> “君子应处木雁之间，当有龙蛇之变。”

<p align="center">
  <img src="./img/flowser.jpg" width="360" alt="花" style="border-radius: 8px; margin-right: 12px;"/>
  <img src="./img/sea.jpg" width="360" alt="海" style="border-radius: 8px;"/>
</p>

---

## 🤝 贡献与社区支持

欢迎提交 Issue、功能建议与 Pull Request！  
如果你喜欢 **hyperscoop**，请为项目点亮一个 ⭐️ **Star**，让更多被 Windows 包管理器折磨的开发者发现它的美好！

<p align="center">
  <a href="https://github.com/Super1Windcloud/hyperscoop">
    <img src="https://api.star-history.com/svg?repos=Super1Windcloud/hyperscoop&type=Date" alt="Star 增长趋势图" width="600"/>
  </a>
</p>

<p align="center">
  Copyright © 2026 <a href="https://github.com/Super1Windcloud">superwindcloud</a>. 遵循 MIT 或 Apache-2.0 开源协议发布。
</p>
