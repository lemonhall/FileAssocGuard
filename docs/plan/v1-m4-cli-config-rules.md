# v1 M4 — CLI 规则管理 + rules.json + check

## Goal

交付可用的规则管理 CLI（`rules list/add/remove`）与 `rules.json` 持久化格式，并把 `check` 作为脚本化入口（exit code 语义固定）。

> 注：当前以 Win11 `HashVersion=1` 的 `UserChoiceLatest capture/replay` 为主线：规则以 “ext -> capture label” 的形式存储；不要求用户理解 `ProgId`。

## PRD Trace

- `REQ-011` `REQ-012` `REQ-013`

## Scope

- 做：`rules.json` schema、读写、规则 CRUD、check、（可选）watch-rules。
- 不做：watch/notify（M5）、sysinfo（M6）。

## Acceptance（硬 DoD）

- `rules add/remove/list`：具备幂等行为与清晰错误（remove 不存在要有明确提示）。
- `check`：
  - 对每条规则输出稳定字段（ext/expected/current/status）。
  - exit code 约定：`0=全 OK`，`2=存在 TAMPERED`，`1=运行错误`。

## Files

- Modify: `crates/fag-cli/src/main.rs`
- Create: `crates/fag-cli/src/rules.rs`
- Runtime: `%APPDATA%\\FileAssocGuard\\rules.json`

## Steps（TDD）

1) **Red — rules schema 单测**  
   - Run: `cargo test -p fag-cli rules::`
   - Expected: FAIL

2) **Green — 实现 rules 读写 + 规则 CRUD**  
   - Run: `cargo test -p fag-cli rules::`
   - Expected: PASS

3) **Green — CLI 接线（rules/check/watch-rules）**  
   - Run: `cargo test`
   - Expected: PASS

4) **Manual Verification（可重复）**  
   - `cargo run -p fag-cli -- rules list`
   - `cargo run -p fag-cli -- check`（观察 exit code：`$LASTEXITCODE`）
   - `cargo run -p fag-cli -- watch-rules --interval 5`

5) **Refactor（仍绿）**  
   - 输出格式集中管理（避免各命令手写不同字段）

## Risks

- JSON 字段命名一旦发布就很难改；先定义 version 字段并保留向后兼容空间。

## Notes

- `snapshot`（从当前系统关联推导规则）本阶段暂不实现：对 `HashVersion=1` 场景，仍建议用户先通过 Windows UI 设置一次，然后 `capture-latest` + `rules add`。
