# v2 M1 — Godot GUI shell (CLI backend)

## Goal

提供一个可以直接跑起来的 Godot GUI 项目（不先纠结 GDExtension），通过调用本仓库的 `fag.exe`（JSON 输出）完成最小闭环：

- 展示 `sysinfo/latest/progids`
- 触发 `capture-latest/apply-latest`
- 触发 `rules add/remove` + `check`

## PRD Trace

- `REQ-100` `REQ-101` `REQ-105`

## Acceptance（硬 DoD）

- `scripts/build-gui.ps1` 成功把 `target\\release\\fag.exe` 复制到 `apps/gui/bin/fag.exe`
- Godot 打开 `apps/gui/project.godot`，运行主场景后：
  - 点击 `Sysinfo` 能显示 JSON
  - 输入 `.mp4` + `vlc`，点击 `Capture/Apply/Rule Add/Check` 能得到可读输出（失败时明确提示原因）

## Verification（可重复）

- `powershell -ExecutionPolicy Bypass -File scripts/build-gui.ps1`
- 手动：Godot 运行 `apps/gui/main.tscn`

## Files

- `apps/gui/project.godot`
- `apps/gui/main.tscn`
- `apps/gui/Main.gd`
- `scripts/build-gui.ps1`

