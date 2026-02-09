# FileAssocGuard v1（Phase 1: Rust CLI）执行计划索引

> PRD: `docs/prd/2026-02-09-fileassocguard.md`  
> Source: `init_prd.md`

## Version Goal

交付 **Phase 1 Rust CLI MVP**：支持规则管理、检测篡改、恢复文件关联、前台守护，并对 Win11 `UserChoiceLatest` 新机制提供检测与指引（不实现新 Hash）。

## Milestones

| Milestone | Plan | DoD（硬验收） | Verification（可重复） | Status |
|---|---|---|---|---|
| M1 | `docs/plan/v1-m1-workspace-registry-read.md` | `cargo test` 全绿；`fag.exe read --ext .mp4` 可读出 `ProgId/Hash/LastWriteTime`（或明确无值） | `cargo test`; `cargo run -p fag-cli -- read --ext .mp4` | done |
| M2 | `docs/plan/v1-m2-hash-algorithm.md` | Hash 算法通过已知向量单测；在真实系统数据上可复算出一致 Hash | `cargo test -p fag-core hash::`（含向量）；附带 `tools/` 或测试辅助读值 | done |
| M3 | `docs/plan/v1-m3-registry-write-restore.md` | `restore` 可写回系统认可的 `ProgId/Hash`（跨分钟自动重试） | `cargo run -p fag-cli -- restore`; 双击/设置验证 + 读取回查 | todo |
| M4 | `docs/plan/v1-m4-cli-config-rules.md` | `snapshot/list/add/remove/check` 可用；JSON 持久化可回归 | `cargo run -p fag-cli -- snapshot ...`; `check` exit code 语义固定 | todo |
| M5 | `docs/plan/v1-m5-watch-notify.md` | `watch` 轮询+自动恢复；可选 Toast 通知；事件落日志 | `cargo run -p fag-cli -- watch --interval 5`; 检查日志输出 | todo |
| M6 | `docs/plan/v1-m6-sysinfo-detection.md` | `sysinfo` 输出 SID/HashVersion/UserChoiceLatest/UCPD，且指引可执行 | `cargo run -p fag-cli -- sysinfo` | todo |
| M7 | `docs/plan/v1-m7-release-hardening.md` | README + 发布产物；最小集成测试；`cargo build --release` 成功 | `cargo test`; `cargo build --release`; 手动 smoke checklist | todo |

## Plan Index

- `docs/plan/v1-m1-workspace-registry-read.md`
- `docs/plan/v1-m2-hash-algorithm.md`
- `docs/plan/v1-m3-registry-write-restore.md`
- `docs/plan/v1-m4-cli-config-rules.md`
- `docs/plan/v1-m5-watch-notify.md`
- `docs/plan/v1-m6-sysinfo-detection.md`
- `docs/plan/v1-m7-release-hardening.md`

## Traceability Matrix（v1）

| Req ID | Plan Coverage | Verification Hook | Evidence（产物路径） |
|---|---|---|---|
| REQ-001 | M1..M7 | `cargo test` + Win11 手动运行 | `docs/plan/evidence/v1/platform.md` |
| REQ-010 | M1 | `cargo run -p fag-cli -- read --ext .mp4` | `docs/plan/evidence/v1/m1-read.md` |
| REQ-011 | M4 | `snapshot` 生成/更新 JSON | `docs/plan/evidence/v1/m4-snapshot.md` |
| REQ-012 | M4 | `add/remove/list` 回归 | `docs/plan/evidence/v1/m4-rules.md` |
| REQ-013 | M4 | `check` 输出 + exit code | `docs/plan/evidence/v1/m4-check.md` |
| REQ-014 | M2,M3 | `cargo test -p fag-core hash::` + `restore` + 回查 read | `docs/plan/evidence/v1/m2-hash.md`, `docs/plan/evidence/v1/m3-restore.md` |
| REQ-015 | M5 | `watch` 轮询恢复 | `docs/plan/evidence/v1/m5-watch.md` |
| REQ-016 | M5 | Toast 可见（开关） | `docs/plan/evidence/v1/m5-toast.md` |
| REQ-017 | M6 | `sysinfo` 输出字段齐全 | `docs/plan/evidence/v1/m6-sysinfo.md` |
| REQ-018 | M6 | `HashVersion=1` 场景输出 ViveTool 指引 | `docs/plan/evidence/v1/m6-latest-guidance.md` |
| REQ-019 | M5 | 日志格式/字段可解析 | `docs/plan/evidence/v1/m5-logs.md` |

## Doc QA Gate（强制）

在开始实现前，必须保证：

- `docs/prd/2026-02-09-fileassocguard.md` 中每条 `REQ-*` 都有可判定验收。
- 本版本每个 `docs/plan/v1-*.md` 都包含 `PRD Trace`（`REQ-*`）。
- 每个计划条目包含可重复命令与预期（至少 “red/green” 的期望失败/通过）。

## Known Deltas（本轮结束仍可能未满足）

- `UserChoiceLatest` 新 Hash（仅检测与引导，不实现）。
- GUI（Godot）全部内容（进入 v2+）。

## Delivery Notes

- CI/推送：当前执行环境中 `git push` 由于 GitHub HTTPS 凭据提示不可交互而失败（`could not read Username for 'https://github.com'`）；需在本机交互式环境完成 push。
