# v1 M5 — watch 守护 + 通知 + 日志

## Goal

提供 `watch` 守护模式：轮询检测、自动恢复、记录事件日志，并在启用时触发 Win11 Toast 通知。

## PRD Trace

- `REQ-015` `REQ-016` `REQ-019`

## Scope

- 做：monitor 轮询、watch 生命周期、事件日志、通知开关与实现。
- 不做：sysinfo（M6）。

## Acceptance（硬 DoD）

- `watch --interval 5`：
  - 每个周期至少执行一次 `check`；发现篡改后执行 `restore` 并记录“恢复成功/失败”事件。
  - 日志记录包含：时间、ext、old_progid、new_progid、action、result。
- 通知：
  - 开关开启时，发生恢复必须触发 Toast（可手动确认）。
  - 开关关闭时不得触发 Toast（可通过日志确认无通知调用）。

## Files

- Create: `crates/fag-core/src/monitor.rs`
- Create: `crates/fag-core/src/notify.rs`
- Modify: `crates/fag-cli/src/main.rs`
- Create: `logs/`（运行时目录；计划中需定义创建策略）

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

5) **Green — 通知实现（可用 `#[cfg(windows)]`）**  
   - Run: `cargo test`（非 Windows 下可跳过或 stub）
   - Expected: Windows 下可见 Toast

6) **Refactor（仍绿）**  
   - 把“事件”作为结构体贯穿（GUI 复用铺路）

## Risks

- Toast API 选型与权限/依赖（可能需要额外 crate/manifest）；先在 DoD 中明确“最小可见通知”标准。

