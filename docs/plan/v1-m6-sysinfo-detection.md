# v1 M6 — sysinfo + UserChoiceLatest/UCPD 检测与指引

## Goal

实现 `sysinfo`：输出用户 SID、HashVersion、UserChoiceLatest 是否启用、UCPD 状态，并在新机制启用时给出**可执行指引**（不依赖外部 exe）。

## PRD Trace

- `REQ-017` `REQ-018`

## Scope

- 做：SID 获取、HashVersion 读取、UCPD 状态检测（至少给出“是否启用/是否影响媒体文件”的结论）、指引输出。
- 不做：自动执行系统开关（只提示；真正的 workaround 在 M8 里提供命令）。

## Acceptance（硬 DoD）

- `cargo run -p fag-cli -- sysinfo` 输出包含字段（字段名固定）：
  - `sid`
  - `hash_version`
  - `user_choice_latest_enabled`（bool）
  - `ucpd_enabled`（bool 或 `unknown`，但必须可判定）
  - `guidance`（当 latest enabled 时包含 `fag win11 disable-userchoicelatest` 指引）

## Files

- Create: `crates/fag-core/src/sysinfo.rs`
- Modify: `crates/fag-cli/src/main.rs`
- Modify: `crates/fag-core/src/lib.rs`

## Steps（TDD）

1) **Red — sysinfo 结构与输出契约测试**  
   - Run: `cargo test -p fag-cli sysinfo_output`
   - Expected: FAIL

2) **Green — 实现 SID 获取（Windows）**  
   - Run: `cargo test`
   - Expected: PASS（非 Windows 可 stub 或编译跳过）

3) **Green — 实现 HashVersion/Latest 检测**  
   - Run: `cargo run -p fag-cli -- sysinfo`
   - Expected: 输出字段齐全

4) **Green — 加入 guidance 逻辑**  
   - 在 latest enabled 的分支输出 `fag win11 disable-userchoicelatest` 指引（无外部 exe）

5) **Refactor（仍绿）**  
   - sysinfo 输出采用结构化（建议 JSON）并在 CLI 层提供可读渲染

## Risks

- UCPD 检测可能需要更深入系统信息；先确保输出“unknown”时也可判定并给出解释。
