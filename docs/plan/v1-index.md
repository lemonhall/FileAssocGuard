# FileAssocGuard v1（Phase 1: Rust CLI）执行计划索引

> PRD: `docs/prd/2026-02-09-fileassocguard.md`  
> Source: `init_prd.md`

## Version Goal

交付 **Phase 1 Rust CLI MVP**：支持规则管理、检测篡改、恢复文件关联、前台守护，并在 `HashVersion=1` 场景下通过 **UserChoiceLatest capture/replay** 方式实现“可恢复”（不逆向新 Hash、不依赖外部 exe）。

## Milestones

| Milestone | Plan | DoD（硬验收） | Verification（可重复） | Status |
|---|---|---|---|---|
| M1 | `docs/plan/v1-m1-workspace-registry-read.md` | `cargo test` 全绿；`fag.exe read --ext .mp4` 可读出 `ProgId/Hash/LastWriteTime`（或明确无值） | `cargo test`; `cargo run -p fag-cli -- read --ext .mp4` | done |
| M2 | `docs/plan/v1-m2-hash-algorithm.md` | Hash 算法通过已知向量单测；在真实系统数据上可复算出一致 Hash | `cargo test -p fag-core hash::`（含向量）；附带 `tools/` 或测试辅助读值 | done |
| M3 | `docs/plan/v1-m3-registry-write-restore.md` | 旧版 `UserChoice` restore：`restore` 可写回系统认可的 `ProgId/Hash`（跨分钟自动重试） | `cargo run -p fag-cli -- restore`; 双击/设置验证 + 读取回查 | blocked (HashVersion=1) |
| M3b | `docs/plan/v1-m3b-userchoicelatest-replay.md` | `HashVersion=1` 场景下：支持 `capture-latest/apply-latest/latest`，可在 VLC / PotPlayer 间来回恢复 | `cargo run -p fag-cli -- latest --ext .mp4`; `cargo run -p fag-cli -- capture-latest ...`; `cargo run -p fag-cli -- apply-latest ...` | done |
| M4 | `docs/plan/v1-m4-cli-config-rules.md` | `rules/check/watch-rules` 可用；`rules.json` 持久化可回归；`check` exit code 语义固定 | `cargo run -p fag-cli -- rules ...`; `cargo run -p fag-cli -- check`; `cargo run -p fag-cli -- watch-rules --interval 5` | done (no snapshot yet) |
| M5 | `docs/plan/v1-m5-watch-notify.md` | `watch/watch-rules` 轮询+自动恢复；事件落日志 | `cargo run -p fag-cli -- watch --interval 5`; `cargo run -p fag-cli -- watch-rules --interval 5` | done (toast deferred) |
| M6 | `docs/plan/v1-m6-sysinfo-detection.md` | `sysinfo` 输出 SID/HashVersion/UserChoiceLatest/UCPD，且指引可执行 | `cargo run -p fag-cli -- sysinfo` | done |
| M7 | `docs/plan/v1-m7-release-hardening.md` | README + 发布产物；最小集成测试；`cargo build --release` 成功 | `cargo test`; `cargo build --release`; 手动 smoke checklist | done |

## Plan Index

- `docs/plan/v1-m1-workspace-registry-read.md`
- `docs/plan/v1-m2-hash-algorithm.md`
- `docs/plan/v1-m3-registry-write-restore.md`
- `docs/plan/v1-m3b-userchoicelatest-replay.md`
- `docs/plan/v1-m4-cli-config-rules.md`
- `docs/plan/v1-m5-watch-notify.md`
- `docs/plan/v1-m6-sysinfo-detection.md`
- `docs/plan/v1-m7-release-hardening.md`

## Traceability Matrix（v1）

| Req ID | Plan Coverage | Verification Hook | Evidence（产物路径） |
|---|---|---|---|
| REQ-001 | M1..M7 | `cargo test` + Win11 手动运行 | `docs/plan/evidence/v1/m7-release.md` |
| REQ-010 | M1 | `cargo run -p fag-cli -- read --ext .mp4` | `docs/plan/evidence/v1/m1-read.md` |
| REQ-011 | M4 | (deferred) `snapshot` | `docs/plan/evidence/v1/m4-rules.md` |
| REQ-012 | M4 | `add/remove/list` 回归 | `docs/plan/evidence/v1/m4-rules.md` |
| REQ-013 | M4 | `check` 输出 + exit code | `docs/plan/evidence/v1/m4-rules.md` |
| REQ-014 | M2,M3 | `cargo test -p fag-core hash::` + `restore` + 回查 read | `docs/plan/evidence/v1/m2-hash.md`, `docs/plan/evidence/v1/m3-restore.md` |
| REQ-015 | M5 | `watch` 轮询恢复 | `docs/plan/evidence/v1/m5-watch.md` |
| REQ-016 | M5 | (deferred) Toast | `docs/plan/evidence/v1/m5-watch.md` |
| REQ-017 | M6 | `sysinfo` 输出字段齐全 | `docs/plan/evidence/v1/m6-sysinfo.md` |
| REQ-018 | M6 | `HashVersion=1` 场景输出 ViveTool 指引 | `docs/plan/evidence/v1/m6-sysinfo.md` |
| REQ-019 | M5 | 日志格式/字段可解析 | `docs/plan/evidence/v1/m5-watch.md` |
| REQ-020 | M3b | `capture-latest/apply-latest/latest` | `docs/plan/evidence/v1/m3b-latest-replay.md` |

## Doc QA Gate（强制）

在开始实现前，必须保证：

- `docs/prd/2026-02-09-fileassocguard.md` 中每条 `REQ-*` 都有可判定验收。
- 本版本每个 `docs/plan/v1-*.md` 都包含 `PRD Trace`（`REQ-*`）。
- 每个计划条目包含可重复命令与预期（至少 “red/green” 的期望失败/通过）。

## Known Deltas（本轮结束仍可能未满足）

- `UserChoiceLatest` 新 Hash 逆向/计算（不做；改用 capture/replay 支持 HashVersion=1）。
- GUI（Godot）全部内容（进入 v2+）。

## Delivery Notes

- v1 累积提交均已推送到 GitHub `main`。
