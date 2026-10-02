<a href='https://postimg.cc/HVXTGZq6' target='_blank'><img src='https://i.postimg.cc/HVXTGZq6/mmexport1677398873855.jpg' border='0' alt='mmexport1677398873855'/></a>
> [!IMPORTANT]
> ## Latest Release 请到 [Github](https://github.com/Super1Windcloud/hyperscoop/releases), [English Readme](./README.en.md)

> [!IMPORTANT]
> ## 运行前请关闭国产杀毒软件 ,微软等各种电脑自带的管家(卡巴斯基除外)
------ 

# HYPERSCOOP(hp)

## 🐼 一个更快,更强, 更精美的  windows 包管理器,By Rust( 继承自 scoop )
--- 

## Feature

- > **多进度条, 多线程 , 丰富色彩, 命令识别自动补全 ......**
- > **为Aria2动态选择合适的分片数和并发线程数, 以达到最速下载,拉爆你的带宽**
- > **支持中英双语国际化 (i18n), 自动适配系统语言并支持手动配置切换**
- > **支持URL直链安装, 无拘无束, 随心所欲 ,肆意驰骋**

## 快速开始,三种方式

### 1. By Scoop

- `scoop bucket add hp https://github.com/Super1Windcloud/hyperscoop_source_bucket.git`
- `scoop  install  -u  -s   hp/hp`

--- 

### 2. By  Powershell

```powershell
irm -useb https://raw.githubusercontent.com/Super1Windcloud/hyperscoop/refs/heads/main/install.ps1 | iex
```
---
### 3. By Cargo (Git)

```bash
cargo install --git https://github.com/super1windcloud/hyperscoop
```

---
### 4. 下载[exe](https://github.com/Super1Windcloud/hyperscoop/releases)使用,并添加到 `$env:Path`

## 使用前提

> `hp b k` 查看官方的bucket列表, `hp b -i` 添加所有bucket ,`hp i 7zip` 用于生命周期脚本 , `hp i aria2` 安装在本地

## Bucket Demo

![bucket.png](./img/bucket.png)

## 🏗 Project Status   (Completed🍻🎉🐉)

| ![](https://i.giphy.com/media/CwfC5Pv6Rtp66h4coK/giphy.gif) |
|:-----------------------------------------------------------:|
|                      Under Maintenance                      |

---

## CLI Features
--- 
![img](./img/cmd1.png)
<!-- ![cmd](https://s1.imagehub.cc/images/2025/05/21/6f39fd471bad23c23d610cdb2daab6a4.png) -->
--- 

## ☑️ TODO (功能全部完成, 尽情使用)

- [x]  Alias
- [x]  autoremove (清理无用的孤儿依赖包)
- [x] Bucket
- [x]  bundle (通过 Hpfile 声明式管理环境依赖: dump / install / check / cleanup)
- [x] cat
- [x] cache
- [x]  checkup (别名: check, doctor, 诊断环境、PATH 长度、断开软链与无效 Shim)
- [x]  cleanup (支持 `--dry-run` / `-n` 预演清理)
- [x]  completion (生成 Shell 自动补全脚本)
- [x]  config
- [x]  deps (查看指定 APP 依赖与树状图)
- [x]  edit (在默认或指定编辑器中直接编辑应用 Manifest)
- [x]  export
- [x]  home
- [x]  import
- [x]  info
- [x]  hold (别名: pin; 解锁: unpin, unhold)
- [x] install (支持 `--dry-run` / `-n`)
- [x]  leaves (查看所有顶层独立应用，即叶子节点)
- [x] list
- [x]  log (查看应用 Manifest 的 Git 提交与变更历史)
- [x] prefix
- [x]  reinstall (别名: re, 一键重新安装应用)
- [x] reset
- [x] search
- [x]  service (别名: services, 后台系统服务管理: list / status / start / stop / restart)
- [x]  shellenv (输出各 Shell 环境配置命令: PowerShell, CMD, Bash, Zsh, Nushell)
- [x] shim
- [x]  size (别名: disk-usage, 分析应用、缓存与数据占用)
- [x] status (别名: outdated)
- [x] uninstall (具备反向依赖防护机制，支持 `--dry-run` / `-n`)
- [x] update
- [x]  uses (反向依赖查询: 查询哪些应用依赖指定软件包)
- [x] which
- [x] merge
- [x] credits

--- 

 

[//]: # ([![sky2.jpg]&#40;https://i.postimg.cc/76yfL7XC/sky2.jpg&#41;]&#40;https://postimg.cc/FfD9WYMm&#41;)
![sky](./img/sky2.jpg)
--- 

- 美是一种选择，甚至是一种放弃，而不是贪婪。
- 君子应处木雁之间，当有龙蛇之变

--- 

