# FileAssocGuard Phase 1 (Rust CLI) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Build the Phase 1 Rust CLI MVP for guarding Windows 11 file associations (detect + restore + watch).

**Architecture:** Cargo workspace with `crates/fag-core` (registry/hash/config/monitor) and `crates/fag-cli` (clap UI). Core logic shared for future Godot GDExtension.

**Tech Stack:** Rust stable (MSVC target), `clap` v4, Windows registry APIs (via `windows` crate), JSON config (`serde`).

## Canonical v1 Plan Docs

- `docs/prd/2026-02-09-fileassocguard.md` (Req IDs)
- `docs/plan/v1-index.md` (milestones + traceability)

## Tasks (bite-sized, execute in order)

### Task 1: Create workspace skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `crates/fag-core/Cargo.toml`
- Create: `crates/fag-core/src/lib.rs`
- Create: `crates/fag-cli/Cargo.toml`
- Create: `crates/fag-cli/src/main.rs`

**Steps:**
1) Add minimal workspace + crates
2) Run: `cargo test` → Expected: PASS (empty tests ok)
3) Commit: `git commit -m "v1: scaffold workspace and crates"` (if git is configured)

### Task 2: Implement registry read + `read` command (M1)

Follow: `docs/plan/v1-m1-workspace-registry-read.md`

**Run:**
- `cargo test -p fag-core`
- `cargo run -p fag-cli -- read --ext .mp4`

### Task 3: Implement old UserChoice hash algorithm (M2)

Follow: `docs/plan/v1-m2-hash-algorithm.md`

**Run:**
- `cargo test -p fag-core`

### Task 4: Implement registry write + `restore` command (M3)

Follow: `docs/plan/v1-m3-registry-write-restore.md`

**Run:**
- `cargo test -p fag-core`
- `cargo run -p fag-cli -- restore ...`

### Task 5: Implement config/rules + `snapshot/list/add/remove/check` (M4)

Follow: `docs/plan/v1-m4-cli-config-rules.md`

**Run:**
- `cargo test`
- `cargo run -p fag-cli -- snapshot --extensions .mp4,.mkv`

### Task 6: Implement `watch` + logging + toast (M5)

Follow: `docs/plan/v1-m5-watch-notify.md`

**Run:**
- `cargo run -p fag-cli -- watch --interval 5`

### Task 7: Implement `sysinfo` (M6)

Follow: `docs/plan/v1-m6-sysinfo-detection.md`

**Run:**
- `cargo run -p fag-cli -- sysinfo`

### Task 8: Release hardening (M7)

Follow: `docs/plan/v1-m7-release-hardening.md`

**Run:**
- `cargo test`
- `cargo build --release`

