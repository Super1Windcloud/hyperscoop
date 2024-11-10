# Commands:

**bucket** 管理scoop 所有buckets

> 子命令 _add|list|known|rm repo_name_
>
> scoop bucket add <repo_name> [<repo_url>]**cat** 显示特定 manifest清单文件内容 , 子命令 `app_name`

**cache** 显示或清理下载缓存

> 子命令 [ show|rm [app(s)] * ]
>
> - scoop cache 等效scoop cache show
>
> - scoop cache rm * 清空安装包缓存
>

**checkup** 检查所有潜在问题

**cleanup** 清理移除旧版本的APP

> ❤️ 子命令 scoop cleanup <app> [options]
>
> - scoop cleanup 等效 scoop cleanup *
>
> - option : -a ,--all 等效于 * -k, --cache 一并移除该app的安装包缓存
>

**config** 获取或设置配置文件

> 🦄 子命令 scoop config [rm] name [value] The scoop configuration file is saved at ~/.config/scoop/config.json.
>
> - 显示所有配置 scoop config
>
> - scoop config <name> <config_value> 配置键值对
>
> - scoop config <name> 获取指定键的值
>
> - scoop config rm <name> 移除指定配置
>

**export** 导出已安装的APP和bucket列表为json格式文件

> 🏵 Usage: scoop export > scoopfile.json

**help** 显示命令帮助信息

**home** 打开指定APP的主页

> 👻 : Usage: scoop home <app>

**import** 导入json文件下载列表中的APP ⚽️ `scoop import <scoopfile.json>`

**info** 显示指定APP的信息 🍷 `scoop info app_name`

> Name : zig Description : General-purpose programming language designed for robustness, optimality, and maintainability. Version : 0.13.0 Bucket : main Website : [https://ziglang.org](https://ziglang.org) License : MIT ([https://spdx.org/licenses/MIT.html](https://spdx.org/licenses/MIT.html)) Updated at : 2024/6/7 16:28:11 Updated by : github-actions[bot] Manifest : A:\Scoop\buckets\main\bucket\zig.json Installed : A:\Scoop\apps\zig\0.13.0 Installed size : 293.8 MB Binaries : zig.exe Suggestions : extras/vcredist2022

**install** 安装指定APP 🐘 _Usage: scoop install <app> [options]_

> - scoop install app , 从bucket 安装
>
> - scoop install gh@2.7.0 , 安装指定版本
>
> - scoop install [https://raw.githubusercontent.com/ScoopInstaller/Main/master/bucket/runat.json](https://raw.githubusercontent.com/ScoopInstaller/Main/master/bucket/runat.json) 从URL安装
>
> - scoop install \path\to\app.json , 从本地manifest文件安装
    >
    >     > option :{
    >     >
    >     > -k, --no-cache 不使用下载安装包缓存
    >  -s, --skip-hash-check 跳过哈希验证
    > >    -u, --no-update-scoop 安装前不更新scoop 和buckets
    > > }
>

**list** 列出已安装的所有app 🌈  `scoop list or scoop list app_name`

**prefix** 打印指定APP的安装目录 🐇 `scoop prefix app_name`

**reset** 切换指定的APP版本, 如果同app存在多版本

> :tada:  :coffee:  scoop reset terraform@0.11.1 指定已安装的版本 , scoop reset terraform #这将切换到最新版本

**search** 搜索可用的APP 🍊 🦉 `scoop search 显示所有可安装的包` , scoop search app_name

**shim** 管理所有的shim快捷方式 🥞 🐼

> - scoop shim add myapp 'D:\path\myapp.exe' 添加对该文件的自定义 快捷方式
>
> - scoop shim rm myapp 移除指定的快捷方式
>
> - scoop shim list 列出所有快捷方式
>
> - scoop shim info myapp 显示指定的快捷方式信息
>


**status** 检查已安装APP是否是最新版本 🎇 , 子命令 🐼` scoop status app_name` 检查指定app

**uninstall** 卸载指定APP 🎅 💩 ☃️ Usage: scoop uninstall <app> [options]

> - scoop uninstall app_name [-p, --purge] 同时移除app所有用户数据和本地配置
>

**update** 更新指定APP或者scoop自身和buckets 🗡 🍹 🎲 🕊 🐬 Usage: scoop update <app> [options]

> - scoop update 更新 scoop版本和 所有 buckets
>
> - scoop update app 更新指定app
>
>
> - > options : -k, --no-cache 不使用下载缓存
    >     >
    >     > - -s, --skip-hash-check 跳过哈希验证 ,
    >     >

    >     >
>

**Which** 打印指定APP的可执行文件路径 🤡 🐸 `scoop Which app_name`

**Merge** 移除不同buckets中冗余的manifest文件 🍻 👑 🎠 📲

> hyperscoop Merge

# Options :

### :panda_face:    install , uninstall , update ,search,info命令的app_name支持指定buckets查找 , 格式为 buckets/app_name

### :panda_face:   -v/--version 打印版本信息_

### :panda_face:  -h/--help 打印帮助信息_

