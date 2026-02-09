# 📋 PRD v0.3：FileAssocGuard（文件关联守卫）

> **版本**：v0.3.0（技术方案修订 - Rust CLI + Godot 4.6 GUI）
> **作者**：柠檬叔
> **日期**：2026-02-09
> **状态**：草稿 v3

---

## 一、项目概述

### 1.1 背景与痛点

在 Windows 11 环境下，部分国产软件（如夸克网盘客户端、某些视频/音乐软件）会在后台**反复篡改系统文件关联**，将 `.mp4`、`.mkv` 等文件的默认打开方式劫持到自家播放器。用户手动修改回来后，这些软件会通过计划任务、开机自启、后台进程等手段再次篡改，形成"改了又被改"的死循环。

### 1.2 产品定位

**FileAssocGuard** 是一款轻量级的 Windows 11 文件关联守护工具：

- **Phase 1**：Rust CLI 原型，跑通核心逻辑（快照、检测、恢复、守护）
- **Phase 2**：将 Rust 核心编译为 GDExtension，用 Godot 4.6 构建 GUI 壳子

### 1.3 目标用户

有一定计算机基础的 Windows 用户，尤其是被流氓软件篡改文件关联困扰的用户。

### 1.4 项目基本信息

| 项 | 值 |
|---|---|
| 项目名 | `file-assoc-guard` |
| 项目目录 | `E:\development\file-assoc-guard` |
| 核心语言 | Rust (stable, MSVC target) |
| GUI 引擎 | Godot 4.6 + godot-rust (gdext) v0.4+ |
| 目标平台 | Windows 11 (x86_64) |

---

## 二、核心概念

### 2.1 文件关联在 Windows 中的存储机制

Windows 文件关联涉及以下注册表路径：

```
# 【关键】用户级别的文件类型选择（Win10/11 最终决定"双击用什么打开"）
HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.<ext>\UserChoice
  ├── ProgId    → 关联的程序标识符（如 "PotPlayer.mp4"）
  └── Hash      → 微软私有 Hash，用于校验 ProgId 的合法性

# 【新增·重要】Win11 A/B 测试中的新保护机制
HKCU\...\FileExts\.<ext>\UserChoiceLatest
  ├── ProgId\ProgId  → 关联的程序标识符
  └── Hash           → 新算法计算的 Hash（包含 machine ID）

# 控制是否启用 UserChoiceLatest 的开关
HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\SystemProtectedUserData\<SID>\AnyoneRead\AppDefaults
  └── HashVersion    → 0=使用旧 UserChoice, 1=使用新 UserChoiceLatest

# 系统级别
HKCR\.<ext>
HKLM\Software\Classes\.<ext>
HKCU\Software\Classes\.<ext>
```

### 2.2 UserChoice Hash 算法（旧版）

已被完整逆向，开源实现参考 [mullerdavid/tools_setfta](https://github.com/mullerdavid/tools_setfta)（MIT 协议，C#）：

```
输入 = toLower(extension + sid + progid + regdate + experience)
其中 experience = "user choice set via windows user experience {d18b6dd5-6124-4341-9318-804003bafa0b}"

Hash = Base64( MicrosoftHash( MD5( 输入 ) ) )
```

其中 `MicrosoftHash` 是微软自定义的一个基于 MD5 结果的二次哈希函数，具体实现见 `setfta.cs`。**本项目将用 Rust 重新实现此算法**。

### 2.3 UserChoiceLatest Hash 算法（新版）

2025 年 4 月起，微软通过 A/B 测试（Feature ID: `43229420` + `27623730`）在 Win11 消费版上逐步推送新机制：

- 新 Hash 包含 **machine ID**，不可跨机器漫游
- 算法已被 SetUserFTA 作者逆向并重新实现（闭源商业工具）
- 算法结构与旧版"有些类似"：包含 user ID、timestamp、association details、machine ID

**应对策略（分层）**：

| 层级 | 策略 | 说明 |
|------|------|------|
| **L1** | 实现旧版 UserChoice Hash | 覆盖大部分 Win11 用户（新机制仍在 A/B 测试阶段） |
| **L2** | 检测 `HashVersion` 值 | 判断当前系统是否启用了 UserChoiceLatest |
| **L3** | 若已启用新机制，提示用户通过 ViveTool 禁用 | `vivetool /disable /id:43229420` + `vivetool /disable /id:27623730` |
| **L4** | 后续版本尝试逆向实现新版 Hash | 作为长期目标，不阻塞 MVP |

### 2.4 UCPD.sys 驱动的影响

微软在 2024 年引入了 `UCPD.sys`（UserChoice Protection Driver），会**阻止特定进程**修改以下类型的关联：`http`、`https`、`.pdf` 等。

**对本项目的影响**：

- 本项目主要保护 `.mp4`、`.mkv`、`.mp3` 等**媒体文件关联**，**不在 UCPD 的保护列表中**，因此不受影响
- 如果用户需要保护 `.pdf` 等被 UCPD 保护的类型，需要先禁用 UCPD 驱动（可在 GUI 中提供引导）

### 2.5 守护机制

```
┌─────────────┐     检测到变化      ┌──────────────┐
│  注册表监控   │ ──────────────→  │  对比快照规则  │
│  (轮询)      │                   │              │
└─────────────┘                   └──────┬───────┘
                                         │
                                    匹配到规则？
                                    ┌────┴────┐
                                    │ Yes     │ No
                                    ▼         ▼
                              ┌──────────┐  ┌──────────┐
                              │ 计算Hash  │  │ 忽略/记录 │
                              │ 写回注册表│  │          │
                              │ 发送通知  │  │          │
                              └──────────┘  └──────────┘
```

---

## 三、功能需求

### 3.1 Phase 1 功能列表（Rust CLI）

| 编号 | 功能 | 优先级 | 说明 |
|------|------|--------|------|
| F01 | **读取文件关联** | P0 | 读取指定扩展名的当前 UserChoice ProgId |
| F02 | **快照当前关联** | P0 | 批量扫描常见扩展名，输出到 config.json |
| F03 | **计算 UserChoice Hash** | P0 | Rust 实现旧版 Hash 算法 |
| F04 | **写入/恢复文件关联** | P0 | 写入 ProgId + 正确的 Hash |
| F05 | **检测篡改** | P0 | 对比当前关联与规则，报告差异 |
| F06 | **watch 守护模式** | P0 | 前台常驻，轮询检测 + 自动恢复 |
| F07 | **Win11 系统通知** | P1 | 恢复后弹出 Toast 通知 |
| F08 | **检测 UserChoiceLatest 状态** | P1 | 检查 HashVersion，提示用户是否需要禁用新机制 |
| F09 | **添加/移除守护规则** | P0 | CLI 命令管理规则 |
| F10 | **配置持久化** | P0 | JSON 文件存储规则 |
| F11 | **日志记录** | P1 | 篡改事件记录到日志文件 |

### 3.2 Phase 2 功能列表（Godot GUI）

| 编号 | 功能 | 优先级 | 说明 |
|------|------|--------|------|
| F20 | **系统托盘常驻** | P0 | 托盘图标 + 右键菜单 |
| F21 | **主界面 - 守护规则列表** | P0 | 可视化展示/编辑守护规则 |
| F22 | **添加规则弹窗** | P0 | 选择扩展名 + 浏览选择程序 |
| F23 | **最近事件面板** | P1 | 展示最近的篡改/恢复事件 |
| F24 | **设置面板** | P1 | 轮询间隔、开机自启、通知开关 |
| F25 | **UCPD/UserChoiceLatest 状态检测** | P1 | 在界面中展示当前系统保护状态，提供操作引导 |
| F26 | **深色模式** | P1 | 跟随系统或默认深色 |

### 3.3 暂不实现（远期）

| 编号 | 功能 | 说明 |
|------|------|------|
| F30 | 逆向实现 UserChoiceLatest 新版 Hash | 需要深入逆向 Windows 二进制 |
| F31 | 识别篡改来源进程 | 通过 ETW 或进程审计 |
| F32 | 导入/导出配置 | 多机同步 |

---

## 四、界面设计

> Phase 2 实现，此处先定义设计规范。

### 4.1 设计原则

- **Win11 风格**：圆角、半透明/毛玻璃质感、Segoe UI Variable 字体风格
- **简洁**：单窗口 + 托盘，不搞多级导航
- **深色优先**：默认深色主题，IT 人友好
- **Godot 实现**：使用 Godot 的 Control 节点 + Theme 系统 + StyleBoxFlat 实现 Win11 风格

### 4.2 系统托盘

```
托盘图标：🛡️ 盾牌样式
    左键单击 → 打开/显示主界面
    右键单击 → 弹出菜单：
        ┌─────────────────┐
        │ ● 守护中          │  ← 状态指示（绿色圆点）
        ├─────────────────┤
        │ 📋 打开主界面      │
        │ ⏸️ 暂停守护       │
        │ 📄 查看日志        │
        ├─────────────────┤
        │ ❌ 退出           │
        └─────────────────┘
```

> 注：Godot 原生不支持系统托盘，需要通过 GDExtension（Rust 侧）调用 Win32 Shell_NotifyIcon API 实现。

### 4.3 主界面

```
┌──────────────────────────────────────────────────────┐
│  🛡️ FileAssocGuard                          ─  □  ✕ │
├──────────────────────────────────────────────────────┤
│                                                      │
│  守护状态：● 运行中                    [⏸ 暂停]      │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │  扩展名     │  当前关联程序        │  操作      │  │
│  ├────────────────────────────────────────────────┤  │
│  │  .mp4       │  PotPlayer           │  ✏️  🗑️  │  │
│  │  .mkv       │  PotPlayer           │  ✏️  🗑️  │  │
│  │  .mp3       │  foobar2000          │  ✏️  🗑️  │  │
│  └────────────────────────────────────────────────┘  │
│                                                      │
│  [+ 添加规则]   [📸 快照当前关联]   [⚙️ 设置]        │
│                                                      │
│  ── 最近事件 ──────────────────────────────────────  │
│  │ 17:05  .mp4 被篡改为 QuarkPlayer → 已恢复 ✅     │  │
│  │ 16:32  .mkv 被篡改为 QuarkPlayer → 已恢复 ✅     │  │
│                                                      │
│  ── 系统状态 ──────────────────────────────────────  │
│  │ UserChoice Hash：✅ 旧版（正常）                  │  │
│  │ UCPD 驱动：⚠️ 已启用（不影响媒体文件）            │  │
│                                                      │
└──────────────────────────────────────────────────────┘
```

---

## 五、技术方案

### 5.1 整体架构

```
┌─────────────────────────────────────────────────────────┐
│                    Godot 4.6 (GUI)                      │
│                                                         │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐             │
│  │ 主界面    │  │ 规则编辑  │  │ 设置面板   │  ← GDScript│
│  │ Scene    │  │ Dialog   │  │ Scene     │             │
│  └────┬─────┘  └────┬─────┘  └─────┬─────┘             │
│       │              │              │                    │
│       └──────────────┼──────────────┘                    │
│                      │                                   │
│              ┌───────▼────────┐                          │
│              │  GDExtension   │  ← godot-rust (gdext)   │
│              │  Bridge Layer  │                          │
│              └───────┬────────┘                          │
│                      │                                   │
├──────────────────────┼──────────────────────────────────┤
│                      │         Rust Core Library         │
│              ┌───────▼────────┐                          │
│              │  lib.rs        │                          │
│              │  (cdylib)      │                          │
│              ├────────────────┤                          │
│              │ registry.rs    │ ← winreg + Win32 API    │
│              │ hash.rs        │ ← UserChoice Hash 算法  │
│              │ monitor.rs     │ ← 轮询守护线程          │
│              │ config.rs      │ ← serde_json            │
│              │ notify.rs      │ ← Win11 Toast 通知      │
│              │ tray.rs        │ ← Shell_NotifyIcon      │
│              └────────────────┘                          │
└─────────────────────────────────────────────────────────┘
```

### 5.2 Phase 1 技术选型（Rust CLI）

| 组件 | 选型 | 理由 |
|------|------|------|
| 语言 | **Rust** (stable, `x86_64-pc-windows-msvc`) | 零运行时、体积小、直接调 Win32 API |
| 注册表操作 | **winreg** crate | Rust 生态最成熟的 Windows 注册表库 |
| UserChoice Hash | **自行实现** | 参考 `mullerdavid/tools_setfta`（MIT）的 C# 源码，用 Rust 重写 |
| MD5 | **md5** crate | Hash 算法依赖 |
| CLI 框架 | **clap** v4 | Rust CLI 标准选择 |
| JSON 配置 | **serde** + **serde_json** | 序列化/反序列化 |
| 系统通知 | **winrt-notification** | Win11 原生 Toast 通知 |
| 日志 | **tracing** + **tracing-subscriber** | 结构化日志 |
| 表格输出 | **comfy-table** | CLI 中漂亮的表格展示 |
| Windows API | **windows** crate (微软官方) | 获取 SID、时间戳等系统信息 |

### 5.3 Phase 2 技术选型（Godot GUI）

| 组件 | 选型 | 理由 |
|------|------|------|
| GUI 引擎 | **Godot 4.6** | 成熟的 UI 系统、Theme 支持、导出为 Windows exe |
| Rust 绑定 | **godot-rust (gdext) v0.4+** | 最低支持 Godot 4.2，4.6 在兼容范围内 |
| 编译产物 | Rust → `.dll` (cdylib) → Godot 通过 `.gdextension` 加载 | 标准 GDExtension 流程 |
| 系统托盘 | Rust 侧通过 `windows` crate 调用 `Shell_NotifyIcon` | Godot 原生不支持托盘，由 Rust 扩展提供 |
| 无边框窗口 | Godot `ProjectSettings` + Win32 `DwmExtendFrameIntoClientArea` | 实现 Win11 风格窗口 |

### 5.4 Cargo Workspace 结构

```
E:\development\file-assoc-guard\
├── Cargo.toml                          # workspace root
├── README.md
│
├── crates/
│   ├── fag-core/                       # 核心库（纯逻辑，无 UI 依赖）
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── registry.rs             # 注册表读写
│   │       ├── hash.rs                 # UserChoice Hash 算法
│   │       ├── monitor.rs              # 轮询守护
│   │       ├── config.rs               # JSON 配置管理
│   │       ├── notify.rs               # Win11 通知
│   │       ├── snapshot.rs             # 快照功能
│   │       └── constants.rs            # 常见扩展名预设
│   │
│   ├── fag-cli/                        # Phase 1: CLI 前端
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs                 # clap CLI，调用 fag-core
│   │
│   └── fag-gdext/                      # Phase 2: GDExtension 桥接层
│       ├── Cargo.toml                  # 依赖 fag-core + godot
│       └── src/
│           ├── lib.rs                  # GDExtension 入口
│           ├── guard_node.rs           # 暴露给 Godot 的节点/类
│           └── tray.rs                 # 系统托盘（Win32）
│
├── godot/                              # Phase 2: Godot 项目
│   ├── project.godot
│   ├── fag-gdext.gdextension          # GDExtension 声明文件
│   ├── scenes/
│   │   ├── main.tscn                   # 主界面
│   │   ├── add_rule_dialog.tscn        # 添加规则弹窗
│   │   └── settings.tscn              # 设置面板
│   ├── scripts/
│   │   ├── main.gd
│   │   ├── add_rule_dialog.gd
│   │   └── settings.gd
│   ├── themes/
│   │   └── dark_win11.tres            # Win11 深色主题
│   └── assets/
│       └── icon.ico
│
├── config.json                         # 运行时配置（守护规则）
└── logs/
    └── guard.log
```

**设计要点**：

- **`fag-core`** 是纯逻辑库，编译为 `rlib`，不依赖任何 UI 框架
- **`fag-cli`** 依赖 `fag-core`，编译为 `exe`，Phase 1 的交付物
- **`fag-gdext`** 依赖 `fag-core` + `godot`，编译为 `cdylib`（.dll），Phase 2 的交付物
- 核心逻辑**只写一次**，CLI 和 GUI 共享同一份代码

### 5.5 CLI 命令设计

```bash
fag.exe <COMMAND>

Commands:
  snapshot    快照当前系统文件关联
  list        列出当前守护规则
  add         添加一条守护规则
  remove      移除一条守护规则
  check       立即检查一次，报告哪些被篡改了
  restore     立即恢复所有被篡改的关联
  watch       进入守护模式（前台常驻，轮询监控 + 自动恢复）
  sysinfo     显示系统状态（UCPD、UserChoiceLatest、当前用户 SID 等）
```

#### 用法示例：

```powershell
# 查看系统状态
fag.exe sysinfo
# 输出：
# 用户 SID:        S-1-5-21-xxxxx-xxxxx-xxxxx-1001
# UCPD 驱动:       已启用（不影响媒体文件关联）
# HashVersion:     0（使用旧版 UserChoice Hash）✅
# UserChoiceLatest: 未启用

# 快照当前关联
fag.exe snapshot --extensions .mp4,.mkv,.avi,.flv,.mp3,.flac

# 查看规则
fag.exe list
# ┌──────────┬─────────────────┬──────────────────────────────┬─────────┐
# │ 扩展名    │ ProgId          │ 程序路径                      │ 状态    │
# ├──────────┼─────────────────┼──────────────────────────────┼─────────┤
# │ .mp4     │ PotPlayer.mp4   │ C:\...\PotPlayerMini64.exe   │ ✅ 正常 │
# │ .mkv     │ PotPlayer.mkv   │ C:\...\PotPlayerMini64.exe   │ ⚠️ 篡改 │
# └──────────┴─────────────────┴──────────────────────────────┴─────────┘

# 立即恢复
fag.exe restore

# 进入守护模式
fag.exe watch --interval 5
# 🛡️ 守护模式已启动，监控 6 个扩展名，间隔 5s
# [17:05:03] ⚠️ .mp4 被篡改: QuarkPlayer → 已恢复为 PotPlayer.mp4 ✅
# [17:05:03] 📢 已发送系统通知
```

### 5.6 GDExtension 桥接层设计（Phase 2）

Rust 侧暴露给 Godot 的类：

```rust
// fag-gdext/src/guard_node.rs

#[derive(GodotClass)]
#[class(base=Node)]
pub struct FileAssocGuard {
    // 内部持有 fag-core 的实例
}

#[godot_api]
impl FileAssocGuard {
    // ---- 规则管理 ----
    #[func] fn load_config(&mut self) -> bool;
    #[func] fn save_config(&self) -> bool;
    #[func] fn get_rules(&self) -> Array<Dictionary>;       // 返回规则列表
    #[func] fn add_rule(&mut self, ext: GString, prog_id: GString, program_path: GString) -> bool;
    #[func] fn remove_rule(&mut self, ext: GString) -> bool;

    // ---- 检测与恢复 ----
    #[func] fn check_all(&self) -> Array<Dictionary>;       // 返回篡改列表
    #[func] fn restore_all(&self) -> i32;                   // 返回恢复数量
    #[func] fn snapshot(&self, extensions: PackedStringArray) -> Array<Dictionary>;

    // ---- 守护模式 ----
    #[func] fn start_watch(&mut self, interval_secs: i32);
    #[func] fn stop_watch(&mut self);
    #[func] fn is_watching(&self) -> bool;

    // ---- 系统信息 ----
    #[func] fn get_sysinfo(&self) -> Dictionary;            // SID, UCPD状态, HashVersion等

    // ---- 系统托盘 ----
    #[func] fn create_tray(&mut self);
    #[func] fn destroy_tray(&mut self);

    // ---- 信号 ----
    #[signal] fn association_tampered(ext: GString, old_progid: GString, new_progid: GString);
    #[signal] fn association_restored(ext: GString, progid: GString);
    #[signal] fn tray_clicked();
    #[signal] fn tray_menu_selected(action: GString);
}
```

Godot 侧（GDScript）使用示例：

```gdscript
# main.gd
extends Control

@onready var guard = FileAssocGuard.new()

func _ready():
    add_child(guard)
    guard.load_config()
    guard.association_tampered.connect(_on_tampered)
    guard.association_restored.connect(_on_restored)
    guard.tray_clicked.connect(_on_tray_clicked)
    guard.create_tray()
    guard.start_watch(5)

func _on_tampered(ext, old_progid, new_progid):
    # 更新 UI 事件列表
    add_event_log("⚠️ %s 被篡改: %s" % [ext, new_progid])

func _on_restored(ext, progid):
    add_event_log("✅ %s 已恢复: %s" % [ext, progid])
```

---

## 六、配置文件格式

```json
{
  "version": "0.1.0",
  "monitor_interval_seconds": 5,
  "show_notification": true,
  "rules": [
    {
      "extension": ".mp4",
      "prog_id": "PotPlayer.mp4",
      "program_path": "C:\\Program Files\\PotPlayer\\PotPlayerMini64.exe",
      "enabled": true
    },
    {
      "extension": ".mkv",
      "prog_id": "PotPlayer.mkv",
      "program_path": "C:\\Program Files\\PotPlayer\\PotPlayerMini64.exe",
      "enabled": true
    }
  ]
}
```

## 七、里程碑

### Phase 1：Rust CLI 原型

| 阶段 | 内容 | 产出 | 预估 |
|------|------|------|------|
| **M1** | Cargo workspace 初始化 + `fag-core` 骨架 + `registry.rs` 封装 | 能读取任意扩展名的当前 UserChoice ProgId 和 Hash | 0.5 天 |
| **M2** | `hash.rs` — 实现 UserChoice Hash 算法 | 给定 ext/sid/progid/timestamp，能算出与系统一致的 Hash | 1~2 天 ⚠️ |
| **M3** | `registry.rs` 写入 + `restore` 命令 | 能正确写回 ProgId + Hash，系统认可（双击验证） | 0.5 天 |
| **M4** | `snapshot` / `list` / `check` / `add` / `remove` 命令 | 完整的规则管理 CLI | 0.5 天 |
| **M5** | `monitor.rs` + `watch` 守护模式 + `notify.rs` 系统通知 | 能后台轮询 + 自动恢复 + 弹 Win11 Toast | 0.5 天 |
| **M6** | `sysinfo` 命令 + UserChoiceLatest 检测 + UCPD 检测 | 用户能了解自己系统的保护状态 | 0.5 天 |
| **M7** | 集成测试 + 修 bug + README + `cargo build --release` | 可发布的 `fag.exe` | 0.5 天 |

**Phase 1 总计：约 4~6 天**

> ⚠️ **M2 是最大风险点**。UserChoice Hash 算法涉及微软私有的二次哈希函数，需要精确复现 C# 实现中的每一步位运算。如果中间某一步对不上，算出来的 Hash 就会被系统拒绝。建议 M2 阶段写充分的单元测试，用已知的 ext/sid/progid/timestamp → Hash 对照组来验证。

### Phase 2：Godot 4.6 GUI

| 阶段 | 内容 | 产出 | 预估 |
|------|------|------|------|
| **M8** | `fag-gdext` crate 初始化 + godot-rust 绑定 + 最小 GDExtension 跑通 | Godot 中能调用 Rust 函数并返回结果 | 1 天 |
| **M9** | `FileAssocGuard` 节点完整 API 暴露 | GDScript 能调用所有核心功能 | 1 天 |
| **M10** | Godot 主界面 Scene + Win11 深色主题 | 规则列表、事件日志、状态栏 | 1~2 天 |
| **M11** | 添加规则弹窗 + 设置面板 | 完整的用户交互流程 | 1 天 |
| **M12** | 系统托盘（Rust 侧 Win32 API） + 最小化到托盘 + 右键菜单 | 托盘常驻体验 | 1 天 |
| **M13** | 开机自启 + 窗口无边框/圆角美化 + 整体打磨 | 接近成品的体验 | 1 天 |
| **M14** | Godot 导出 Windows exe + 集成测试 + 修 bug | 可发布的 `FileAssocGuard.exe` | 1 天 |

**Phase 2 总计：约 7~9 天**

### 总览时间线

```
Week 1          Week 2          Week 3          Week 4
─────────────── ─────────────── ─────────────── ───────────────
Phase 1                         Phase 2
M1 M2 ████ M3  M4 M5 M6 M7     M8 M9 M10██    M11 M12 M13 M14
      ↑                                ↑
      Hash算法                         Godot主界面
      (风险点)                         (工作量最大)
```

---

## 八、UserChoice Hash 算法详细说明

> 本节是 Phase 1 M2 的核心参考，单独拎出来详细说明。

### 8.1 算法输入

```
input_string = toLower(
    extension          // 如 ".mp4"
    + user_sid         // 如 "S-1-5-21-xxxxx-xxxxx-xxxxx-1001"
    + prog_id          // 如 "PotPlayer.mp4"
    + reg_date_time    // FILETIME 格式，取 UserChoice 键的最后写入时间，精度截断到分钟
    + experience       // 固定字符串: "user choice set via windows user experience {d18b6dd5-6124-4341-9318-804003bafa0b}"
)
```

### 8.2 算法步骤

```
1. 将 input_string 编码为 UTF-16LE 字节序列（含 null terminator）
2. 计算 MD5(utf16le_bytes) → 得到 16 字节的 md5_hash
3. 将 md5_hash 视为 4 个 little-endian uint32: dw0, dw1, dw2, dw3
4. 将 input_string 的 UTF-16LE 编码（不含 null terminator）视为 uint32 数组
   （如果字节数不是 4 的倍数，末尾补 0）
5. 执行微软自定义的 "ShiftHash" 循环：
   - 初始化: hash_lo = dw0 ^ dw2, hash_hi = dw1 ^ dw3
   - 每次取 2 个 uint32 (word0, word1)，执行一系列位移、乘法、异或操作
   - 具体的魔数和位移量见 setfta.cs 源码
6. 最终得到 hash_lo (uint32) 和 hash_hi (uint32)
7. 拼接为 uint64: result = (hash_hi << 32) | hash_lo
8. 将 result 的 8 字节（little-endian）进行 Base64 编码
9. 输出即为 UserChoice 的 Hash 值
```

### 8.3 关键注意事项

| 事项 | 说明 |
|------|------|
| **时间戳精度** | `reg_date_time` 必须是 UserChoice 键的**最后写入时间**，且需要**截断到分钟**（秒和更小单位清零）。这意味着写入 ProgId 和计算 Hash 必须在**同一分钟内**完成，否则 Hash 会失效 |
| **时间戳获取** | 需要调用 `RegQueryInfoKeyW` 获取键的 `lpftLastWriteTime`，而不是用系统当前时间 |
| **操作顺序** | 正确的顺序是：① 删除 UserChoice 键 → ② 重新创建 UserChoice 键 → ③ 立即读取键的写入时间 → ④ 用该时间计算 Hash → ⑤ 写入 ProgId 和 Hash |
| **SID 获取** | 通过 `GetTokenInformation` + `ConvertSidToStringSidW` 获取当前用户 SID |
| **大小写** | 整个 input_string 必须 `toLower()`，包括 SID 中的字母 |
| **溢出行为** | ShiftHash 中的乘法必须是 **wrapping_mul**（允许溢出截断），Rust 中需要显式使用 `.wrapping_mul()` / `.wrapping_add()` |

### 8.4 验证策略

```rust
#[cfg(test)]
mod tests {
    // 用已知的输入 → 输出对照组验证算法正确性
    // 方法：在一台干净的 Win11 上手动设置某个文件关联，
    //       然后读取 UserChoice 中的 ProgId、Hash、键写入时间、用户 SID，
    //       用我们的算法重新计算 Hash，验证是否一致
  
    #[test]
    fn test_known_hash_vector_1() {
        let ext = ".mp4";
        let sid = "s-1-5-21-xxxxxxxxx-xxxxxxxxx-xxxxxxxxx-1001";
        let prog_id = "potplayer.mp4";
        let timestamp = "01da..."; // FILETIME hex, 截断到分钟
        let expected_hash = "xxxxxxxx=";
      
        let computed = compute_user_choice_hash(ext, sid, prog_id, timestamp);
        assert_eq!(computed, expected_hash);
    }
}
```

---

## 九、风险与应对

| 风险 | 等级 | 应对 |
|------|------|------|
| **UserChoice Hash 算法实现** | 🔴 高 | 参考 `mullerdavid/tools_setfta`（MIT）C# 源码逐行翻译为 Rust；写充分的单元测试用已知向量验证；**不接受调用外部 exe 作为替代** |
| **UserChoiceLatest 新机制** | 🟡 中 | Phase 1 先检测并提示用户；若已启用新机制，引导用户通过 ViveTool 禁用（`vivetool /disable /id:43229420`）；长期目标是逆向实现新版 Hash |
| **写入时间窗口** | 🟡 中 | 删除键 → 创建键 → 读时间 → 算 Hash → 写值，整个流程必须在同一分钟内完成；代码中加入重试逻辑：如果跨分钟了就重新来一次 |
| **管理员权限** | 🟡 中 | 修改 UserChoice 需要管理员权限；CLI 阶段用管理员终端运行；GUI 阶段在 manifest 中声明 `requireAdministrator` |
| **godot-rust 与 Godot 4.6 兼容性** | 🟡 中 | gdext v0.4+ 支持 Godot 4.2+，4.6 在范围内；但需要关注 `api-custom` feature 的已知 issue，必要时 pin 到特定 commit |
| **Godot 系统托盘** | 🟡 中 | Godot 原生不支持系统托盘；由 Rust 侧通过 `windows` crate 调用 `Shell_NotifyIcon` Win32 API 实现，通过信号通知 Godot 侧 |
| **杀毒误报** | 🟢 低 | Rust 编译的原生 exe 比 PyInstaller 打包好很多，误报概率低 |

