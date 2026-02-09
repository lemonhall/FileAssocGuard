# FileAssocGuard (Phase 1: Rust CLI)

> 当前状态：仅实现到 Phase 1 的早期命令（read/progids/restore）。

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

### 3) 恢复/切换 `.mp4` 关联

```powershell
# 自动从候选 ProgId 里挑一个包含 "vlc" 的
cargo run -p fag-cli -- restore --ext .mp4 --to vlc

# 或指定明确 ProgId
cargo run -p fag-cli -- restore --ext .mp4 --progid VLC.mp4
```

## Win11 新机制（HashVersion=1）怎么处理？

你的系统如果启用了 `HashVersion=1`（UserChoiceLatest），那么我们目前的“旧算法写回”会被系统拒绝。

这部分的目标是：实现 **UserChoiceLatest 新 Hash** 的原生支持（不依赖外部 exe）。在完成之前：

- `restore` 会明确报错提示“未实现”；
- 临时 workaround 是用 ViveTool 禁用相关 Feature ID 并重启（见命令输出）。
