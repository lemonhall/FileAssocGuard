# v2 — Phase 2 Godot GUI（追溯占位，待拆分）

## Goal

把 Phase 1 的 Rust 核心能力通过 GDExtension 暴露给 Godot，并实现 Win11 风格 GUI（托盘常驻 + 单窗口交互）。

## PRD Trace

- `REQ-100` `REQ-101` `REQ-102` `REQ-103` `REQ-104` `REQ-105` `REQ-106`

## Scope

- 做：`fag-gdext` crate（godot-rust gdext）+ Godot 项目目录（`godot/`）+ UI/托盘/交互。
- 不做：新增核心逻辑（核心逻辑优先在 `fag-core` 完成并复用）。

## Acceptance（硬 DoD，v2 细化时必须拆成可跑的测试/命令）

- Godot 可加载扩展并调用 Rust API（至少能 `read/check/restore` 走通一条路径）。
- 托盘常驻：有图标、菜单、点击能显示/隐藏主窗口。
- 主界面：规则列表（增删改）、事件面板、设置面板、系统状态面板（latest/UCPD）。
- 默认深色主题，符合 Win11 风格（圆角、字体、间距统一）。

## Files（预计）

- Create: `crates/fag-gdext/`
- Create: `godot/`
- Modify: `crates/fag-core/`（仅为公开 API 调整，不引入 UI 依赖）

## Steps

在 v2 执行前先把本文件拆成：`v2-m8-*` ... `v2-m14-*`，每个文件按塔山循环补齐 `red/green/refactor` 与验证命令。

