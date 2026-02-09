# v1 M7 — 集成测试 + 修 bug + README + Release

## Goal

把 Phase 1 变成“可发布的 fag.exe”：补齐文档、最小集成测试、稳定输出与打包流程。

## PRD Trace

- `REQ-001` `REQ-019`（发布态需要可信日志/输出）

## Scope

- 做：README、安装/使用说明、最小集成测试策略、Release 构建指令与产物命名。
- 不做：GUI（Phase 2）。

## Acceptance（硬 DoD）

- `cargo test` 全绿（允许 `#[ignore]` 的管理员集成测试，但必须在 README 写清楚运行方式）。
- `cargo build --release` 成功，产物为 `target/release/fag.exe`（或最终命名一致）。
- README 包含：
  - 需要管理员权限的说明
  - 常见问题：UserChoiceLatest 启用时的处理方式
  - 示例命令与输出

## Files

- Create/Modify: `README.md`
- Create: `docs/plan/evidence/v1/`（放验证证据/截图/输出摘要）
- Create: `crates/fag-cli/tests/`（若采用 CLI 级集成测试）

## Steps（TDD）

1) **Red — CLI 集成测试最小样例**  
   - Run: `cargo test -p fag-cli`
   - Expected: FAIL

2) **Green — 实现/调整输出与 exit code 以满足测试**  
   - Run: `cargo test -p fag-cli`
   - Expected: PASS

3) **Green — Release 构建检查**  
   - Run: `cargo build --release`
   - Expected: 成功

4) **Refactor（仍绿）**  
   - 输出/错误处理统一（避免各命令各自 fmt）

## Risks

- 由于管理员权限与系统状态差异，集成测试需要可选择性执行；必须避免“测试只能在作者电脑跑”。

