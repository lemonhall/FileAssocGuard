# v1 M5 — watch 守护 + 日志（Toast 延后）

## Goal

提供 `watch` 守护模式：轮询检测、自动恢复、记录事件日志。

> 注：Win11 Toast（非打包桌面程序）通常涉及 AUMID/快捷方式注册等，容易引入复杂度；为尽快完成 Phase 1 并进入 Godot GUI，本阶段先把日志与守护闭环做硬，Toast 延后到 GUI（v2+）或后续 v1.x。

## PRD Trace

- `REQ-015` `REQ-016` `REQ-019`

## Scope

- 做：monitor 轮询、watch 生命周期、事件日志、通知开关与实现。
- 不做：sysinfo（M6）。

## Acceptance（硬 DoD）

- `watch --interval 5` / `watch-rules --interval 5`：
  - 每个周期至少执行一次 `check`；发现篡改后执行 `restore` 并记录“恢复成功/失败”事件。
  - 日志记录包含：时间、ext、old_progid、new_progid、action、result。
- 日志默认路径：`%APPDATA%\\FileAssocGuard\\guard.log`（JSON lines）。

## Files

- Modify: `crates/fag-cli/src/main.rs`
- Create: `crates/fag-cli/src/logging.rs`

## Steps（TDD）

1) **Red — monitor 循环的“可测试”抽象**  
   - 通过注入 clock/sleeper 或“单步 tick”函数让逻辑可测
   - Run: `cargo test -p fag-core monitor`
   - Expected: FAIL

2) **Green — 实现 watch 单步 tick：check→restore→emit event**  
   - Run: `cargo test -p fag-core monitor`
   - Expected: PASS

3) **Red — 日志格式测试（稳定可解析）**  
   - Run: `cargo test -p fag-core log_format`
   - Expected: FAIL

4) **Green — 实现日志写入 + CLI 接线**  
   - Run: `cargo run -p fag-cli -- watch --interval 5`
   - Expected: 持续运行并写日志

5) **Refactor（仍绿）**  
   - 把“事件”作为结构体贯穿（GUI 复用铺路）

## Risks

- 日志量：watch 的 OK 事件可能很频繁；建议只记录篡改/恢复事件（或未来加采样/滚动）。
