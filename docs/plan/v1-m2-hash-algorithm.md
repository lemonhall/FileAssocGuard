# v1 M2 — 实现旧版 UserChoice Hash（高风险）

## Goal

在 Rust 中 1:1 复现旧版 `UserChoice` Hash 算法，并用**已知向量**与**真实系统数据**双重验证。

## PRD Trace

- `REQ-014`（恢复依赖 Hash）

## Scope

- 做：实现 hash 计算模块、单元测试向量、（可选）工具函数用于从系统采集验证数据。
- 不做：写入/恢复流程（M3）、UserChoiceLatest 新 Hash（非目标）。

## Acceptance（硬 DoD）

- `cargo test -p fag-core` 中包含：
  - ≥ 3 组“已知输入→期望 Base64 Hash”向量测试（跨不同 ext/progid/sid/timestamp）。
  - 1 个“真实系统采集样本”测试（允许 `#[ignore]`，但必须可复现：给出采集步骤与样本文件路径）。
- 对于实现细节：所有整数运算使用 wrapping（`wrapping_add/mul` 等）确保与 C# 溢出行为一致。

## Files

- Create: `crates/fag-core/src/hash.rs`
- Modify: `crates/fag-core/src/lib.rs`
- Test: `crates/fag-core/src/hash.rs`（或 `crates/fag-core/tests/hash_vectors.rs`）

## Steps（TDD）

1) **Red — 写向量测试骨架**  
   - 写 `compute_user_choice_hash(ext, sid, prog_id, last_write_time_truncated_to_minute) -> String`
   - Run: `cargo test -p fag-core hash_vectors`
   - Expected: FAIL（未实现）

2) **Green — 实现 UTF-16LE/MD5 输入准备**  
   - Run: `cargo test -p fag-core hash_vectors`
   - Expected: 仍 FAIL（ShiftHash 未实现或数值不匹配）

3) **Green — 实现 ShiftHash / MicrosoftHash 主体**  
   - Run: `cargo test -p fag-core hash_vectors`
   - Expected: PASS（至少向量测试全绿）

4) **Green — 加“真实系统样本”可复现路径**  
   - 添加 `README`/测试注释：如何读取 ext/sid/progid/last_write_time/hash 并落盘为 fixture
   - Run: `cargo test -p fag-core -- --ignored`
   - Expected: PASS（在包含 fixture 的机器上）

5) **Refactor（仍绿）**  
   - 分离：字符串拼接/编码、MD5、ShiftHash 核心循环
   - 保持向量测试稳定

## Risks

- 魔数/位移/端序/补齐规则任何一处偏差都会导致 Hash 失效；必须以向量测试为锚。

