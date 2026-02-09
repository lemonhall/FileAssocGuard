# v2 — Phase 2 Godot GUI（先用 CLI 后端跑通 UI）

## Goal

尽快交付可用 GUI：先用 Godot 直接调用 `fag.exe`（本仓库内 Rust CLI，JSON 输出）作为后端跑通 UI；后续再把核心能力迁移到 GDExtension（减少进程间调用）。

## PRD Trace

- `REQ-100` `REQ-101` `REQ-102` `REQ-103` `REQ-104` `REQ-105` `REQ-106`

## Scope

- 做：Godot 项目目录（`apps/gui/`）+ UI/交互；调用 `apps/gui/bin/fag.exe`（由 `scripts/build-gui.ps1` 构建复制）。
- 预留：后续新增 `fag-gdext`（godot-rust gdext）替换 CLI 后端。
- 不做：新增核心逻辑（核心逻辑优先在 `fag-core` 完成并复用）。

## Acceptance（硬 DoD，v2 细化时必须拆成可跑的测试/命令）

- Godot 可通过 `OS.execute` 调用 `fag.exe` 并解析 JSON（至少能 `sysinfo/latest/capture/apply/rules/check` 走通一条路径）。
- 托盘常驻：有图标、菜单、点击能显示/隐藏主窗口。
- 主界面：规则列表（增删改）、事件面板、设置面板、系统状态面板（latest/UCPD）。
- 默认深色主题，符合 Win11 风格（圆角、字体、间距统一）。

## Files（预计）

- Create: `apps/gui/`
- Create: `scripts/build-gui.ps1`
- (Later) Create: `crates/fag-gdext/`

## Steps

在 v2 执行前先把本文件拆成：`v2-m1-*` ...，每个文件按塔山循环补齐 `red/green/refactor` 与验证命令。
