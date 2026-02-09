# v1 M1 — Workspace 初始化 + Registry Read

## Goal

建立 Rust workspace 骨架，并实现最小可用的“读取某扩展名当前 UserChoice（ProgId/Hash/LastWriteTime）”能力。

## PRD Trace

- `REQ-001` `REQ-010`

## Scope

- 做：初始化 workspace（`fag-core` + `fag-cli`），封装注册表读取，提供 `read` 子命令。
- 不做：写入/恢复（留到 M3）、hash 计算（留到 M2）、规则/配置（留到 M4）。

## Acceptance（硬 DoD）

- `cargo test` 通过（至少包含 `fag-core` 的单元测试占位 + `fag-cli` 的 clap 解析测试/快照）。
- `cargo run -p fag-cli -- read --ext .mp4`：
  - 若存在关联：输出包含 `Ext/ProgId/Hash/LastWriteTime` 四项（字段名固定，便于脚本解析）。
  - 若不存在：输出包含可判定状态（例如 `NOT_SET`），且 exit code = 0（读取成功但无值）。

## Files

- Create: `Cargo.toml`
- Create: `crates/fag-core/Cargo.toml`
- Create: `crates/fag-core/src/lib.rs`
- Create: `crates/fag-core/src/registry.rs`
- Create: `crates/fag-cli/Cargo.toml`
- Create: `crates/fag-cli/src/main.rs`

## Steps（TDD）

1) **Red — 写测试（registry read 合约）**  
   - Test: `crates/fag-core/src/registry.rs`（或 `crates/fag-core/tests/registry_read.rs`）  
   - Run: `cargo test -p fag-core`  
   - Expected: FAIL（函数/类型未实现）

2) **Green — 最小实现**  
   - 实现 `fag_core::registry::read_user_choice(ext) -> Result<Option<UserChoice>, Error>`
   - `UserChoice` 至少包含：`prog_id: Option<String>`, `hash: Option<String>`, `last_write_time: SystemTime`（或 FILETIME 包装）

3) **Green — CLI 接入**  
   - 实现 `fag.exe read --ext <.ext>` 调用 core 并输出稳定字段
   - Run: `cargo run -p fag-cli -- read --ext .mp4`
   - Expected: 命令可运行，输出格式稳定

4) **Refactor（仍绿）**  
   - 将注册表路径/键名常量集中（避免散落字符串）
   - 保持 `cargo test` 全绿

## Risks

- Win11 注册表路径存在差异（需以实际读取结果为准）；先实现“读到什么就打印什么”，不做推断。

