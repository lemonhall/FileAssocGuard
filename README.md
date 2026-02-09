# FileAssocGuard (Phase 1: Rust CLI)

> 当前状态：已支持 Win11 `HashVersion=1`（`UserChoiceLatest`）的 **capture/replay** 恢复（不依赖外部 exe）。

## 为什么你的 `cargo run` 会报 “找不到 Cargo.toml”？

请确认你在仓库根目录运行，且当前分支包含 Rust workspace（`Cargo.toml` 在根目录）。

## 命令速查

### 1) 查看当前 `.mp4` 关联（UserChoice）

```powershell
cargo run -p fag-cli -- read --ext .mp4
```

### 2) 列出 `.mp4` 可用的 ProgId 候选（用于 VLC / PotPlayer）

```powershell
cargo run -p fag-cli -- progids --ext .mp4
```

### 3) Win11 新机制（HashVersion=1）：capture / apply（推荐）

```powershell
# 看当前 UserChoiceLatest（含 effective_progid）
cargo run -p fag-cli -- latest --ext .mp4

# 先在 Windows 设置里把 .mp4 默认应用切到 VLC 一次，然后捕获
cargo run -p fag-cli -- capture-latest --ext .mp4 --name vlc

# 再切到 PotPlayer 一次，然后捕获
cargo run -p fag-cli -- capture-latest --ext .mp4 --name potplayer

# 之后就可以在两者之间来回恢复（不需要知道 ProgId/Hash）
cargo run -p fag-cli -- apply-latest --ext .mp4 --name vlc
cargo run -p fag-cli -- apply-latest --ext .mp4 --name potplayer

# 查看当前已保存的标签
cargo run -p fag-cli -- captures --ext .mp4
```

### 4) 守护（只守 `.mp4`，最快能用）

```powershell
# 每 5 秒检查一次，如果被篡改就自动 apply 回去（Ctrl+C 停止）
cargo run -p fag-cli -- watch --ext .mp4 --name vlc --interval 5
```

## captures.json 在哪？

默认写入：`%APPDATA%\\FileAssocGuard\\captures.json`（建议自行备份）。
