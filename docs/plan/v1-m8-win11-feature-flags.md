# v1 M8 — Win11 `HashVersion=1` workaround（无外部 exe）

## Context

在部分 Win11 机器上（`HashVersion=1` / `UserChoiceLatest` 启用），直接写入 `UserChoiceLatest` 的 `ProgId/Hash` 可能会被系统忽略/回滚，表现为守护日志出现 `REJECTED`，即使注册表值“写进去”，实际默认程序仍不改变。

本里程碑不做“新版 Hash 逆向计算”，而是提供一个**可审计、无外部 exe** 的 workaround：

- 通过调用 `ntdll.dll` 的 feature configuration API（`RtlSetFeatureConfigurations`），禁用与 `UserChoiceLatest` A/B 推送相关的 feature IDs；
- 用户手动重启后，期望系统回退到旧机制（`HashVersion` 变为 0），此时可继续使用 legacy `restore`（项目已实现旧版 Hash）。

> 注意：这是“开关系统 feature”的高风险操作，必须让用户明确知情；默认不自动执行，只提供命令。

## PRD Trace

- REQ-018（HashVersion=1 场景下提供可执行指引 / workaround）

## Deliverables

- 新增 core 模块：`crates/fag-core/src/features.rs`
  - `query_feature_configuration(...)`
  - `set_feature_state(...)`
- CLI 新增命令：
  - `fag features status --id <id> [--type <boot|runtime>]`
  - `fag features set --id <id> --state <default|disabled|enabled> [--type <boot|runtime>]`
  - `fag win11 disable-userchoicelatest`（批量禁用两个 feature id，提示需要重启）
- `fag sysinfo` 输出 guidance 更新：指向内置命令（不再要求外部 exe）

## DoD（硬验收）

1. `cargo test` 全绿
2. `features status` 能在普通权限下运行，返回 JSON（不崩溃）
3. `win11 disable-userchoicelatest` 命令存在、能输出结构化 JSON；若权限不足，应该明确报错（exit code=1），不 silent fail

## Verification（可重复）

1) 运行单测

- Command: `cargo test`
- Expected: PASS

2) 查询当前 feature 状态（示例：runtime）

- Command: `cargo run -p fag-cli -- features status --id 43229420 --type runtime`
- Expected: 输出 JSON，包含 `enabled_state` 字段（`default|disabled|enabled` 之一）

3) 执行 workaround（需要管理员权限；并通常需要重启）

- Command: `cargo run -p fag-cli -- win11 disable-userchoicelatest`
- Expected: 输出 JSON：`reboot_required=true`；`updates[].*_ok` 可能为 `false`（权限不足时），此时命令应返回非 0 exit code 并在 stderr 说明原因

## Notes / Risks

- feature flags 可能随系统版本变化；本里程碑只保证“提供工具/命令”，不承诺对所有 Win11 build 都能回退成功。
- 这条路的最终目标是让项目在“无法实现新版 Hash”的情况下仍能落地 Phase 1 的“自动恢复”能力。

