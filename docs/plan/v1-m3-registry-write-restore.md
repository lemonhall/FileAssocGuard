# v1 M3 — Registry Write + restore 命令

## Goal

实现“按守护规则写回 UserChoice（ProgId+Hash）且系统认可”，并处理“同一分钟窗口”与重试。

## PRD Trace

- `REQ-014`

## Scope

- 做：`registry.rs` 写入/删除/重建 UserChoice；`restore` CLI；跨分钟重试。
- 不做：规则管理（M4）、守护 watch（M5）、通知（M5）。

## Acceptance（硬 DoD）

- `cargo run -p fag-cli -- restore --ext .mp4 --progid PotPlayer.mp4`（具体参数以最终 CLI 设计为准）：
  - 写入后立刻 `read` 回查，`ProgId/Hash` 与预期一致。
  - 若写入跨分钟导致 hash 失效：自动重试（至少 1 次）并在日志/输出标明“跨分钟重试”。
- 单测覆盖“分钟截断”逻辑与“重试触发条件”（可通过注入 clock 抽象实现）。

## Files

- Modify: `crates/fag-core/src/registry.rs`
- Modify: `crates/fag-cli/src/main.rs`
- Test: `crates/fag-core/tests/`（写入相关可使用 `#[ignore]` 的集成测试，需要管理员时明确标注）

## Steps（TDD）

1) **Red — 写分钟截断与重试策略测试**  
   - Run: `cargo test -p fag-core retry`
   - Expected: FAIL

2) **Green — 实现“删除→创建→读 last_write_time→算 hash→写值”流程**  
   - Run: `cargo test -p fag-core retry`
   - Expected: PASS

3) **Green — CLI `restore` 走通最小路径**  
   - Run: `cargo run -p fag-cli -- restore ...`
   - Expected: 成功输出恢复数量与每条结果（OK/FAIL + reason）

4) **Refactor（仍绿）**  
   - 把“写入事务”封装为单函数，保证顺序不被 future 改坏

## Risks

- 需要管理员权限；测试必须显式区分“纯逻辑单测”与“管理员集成测试”。

