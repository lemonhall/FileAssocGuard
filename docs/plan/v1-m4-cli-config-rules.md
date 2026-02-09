# v1 M4 — CLI 规则管理 + config.json + check/snapshot

## Goal

交付可用的规则管理 CLI（`snapshot/list/add/remove/check`）与 JSON 持久化格式，并把 `check` 作为脚本化入口（exit code 语义固定）。

## PRD Trace

- `REQ-011` `REQ-012` `REQ-013`

## Scope

- 做：`config.json` schema、读写、规则 CRUD、snapshot、check。
- 不做：watch/notify（M5）、sysinfo（M6）。

## Acceptance（硬 DoD）

- `snapshot`：给定扩展名列表后，配置文件包含每条规则（ext→ProgId）。
- `add/remove/list`：具备幂等行为与清晰错误（add 已存在、remove 不存在）。
- `check`：
  - 对每条规则输出稳定字段（ext/expected/current/status）。
  - exit code 约定：`0=全 OK`，`2=存在 TAMPERED`，`1=运行错误`。

## Files

- Create: `crates/fag-core/src/config.rs`
- Create/Modify: `crates/fag-core/src/snapshot.rs`
- Modify: `crates/fag-cli/src/main.rs`
- Create: `config.json`（样例，可放 `examples/config.json`，避免污染运行态）

## Steps（TDD）

1) **Red — config schema 单测**  
   - Run: `cargo test -p fag-core config`
   - Expected: FAIL

2) **Green — 实现 config 读写 + 规则 CRUD**  
   - Run: `cargo test -p fag-core config`
   - Expected: PASS

3) **Red — check 输出与 exit code 测试（CLI 层）**  
   - Run: `cargo test -p fag-cli check_exit_code`
   - Expected: FAIL

4) **Green — CLI 接线（snapshot/list/add/remove/check）**  
   - Run: `cargo test -p fag-cli`
   - Expected: PASS

5) **Refactor（仍绿）**  
   - 输出格式集中管理（避免各命令手写不同字段）

## Risks

- JSON 字段命名一旦发布就很难改；先定义 version 字段并保留向后兼容空间。

