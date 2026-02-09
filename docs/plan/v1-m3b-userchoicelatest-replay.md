# v1 M3b — HashVersion=1：UserChoiceLatest capture/replay

## Goal

在 Windows 11 启用 `UserChoiceLatest`（常见于 `HashVersion=1`）的情况下，不逆向新 Hash、也不依赖外部 exe，仍能做到“可恢复”：

- 读取当前 `UserChoiceLatest`（`latest`）
- 捕获一份系统当前认可的 `(ProgId + Hash)` 并存档（`capture-latest`）
- 在需要时回放写回（`apply-latest`）

## PRD Trace

- `REQ-020`

## Scope

- 做：新增 `latest/capture-latest/apply-latest/captures` CLI；本地存档 `captures.json`；写注册表后通知系统刷新关联。
- 不做：计算/逆向新 Hash；不做自动调用 Windows UI；不做守护 watch（M5）。

## Acceptance（硬 DoD）

- `cargo test` 全绿（含 `captures.json` 读写单测）。
- 手动验证（以 `.mp4` 为例）：
  1) 在 Windows 设置把 `.mp4` 默认应用设置为 VLC。
  2) `cargo run -p fag-cli -- capture-latest --ext .mp4 --name vlc` 保存成功。
  3) 在 Windows 设置把 `.mp4` 默认应用设置为 PotPlayer。
  4) `cargo run -p fag-cli -- capture-latest --ext .mp4 --name potplayer` 保存成功。
  5) `cargo run -p fag-cli -- apply-latest --ext .mp4 --name vlc` 能恢复到 VLC（用 `latest` 的 `effective_progid` 验证）。
  6) `cargo run -p fag-cli -- apply-latest --ext .mp4 --name potplayer` 能恢复到 PotPlayer（同上验证）。

## Verification（可重复）

- `cargo run -p fag-cli -- latest --ext .mp4`
- `cargo run -p fag-cli -- capture-latest --ext .mp4 --name vlc`
- `cargo run -p fag-cli -- apply-latest --ext .mp4 --name vlc`
- `cargo run -p fag-cli -- captures --ext .mp4`

## Files

- Modify: `crates/fag-core/src/registry.rs`
- Modify: `crates/fag-cli/src/main.rs`
- Add: `crates/fag-cli/src/captures.rs`
- Evidence: `docs/plan/evidence/v1/m3b-latest-replay.md`

## Risks / Notes

- 新机制的 `Hash` 可能随系统更新/策略变化而失效；若 `apply-latest` 不生效，重新在系统设置里设置一次并重新 `capture-latest`。
- 存档默认位置：`%APPDATA%\\FileAssocGuard\\captures.json`（建议备份）。

