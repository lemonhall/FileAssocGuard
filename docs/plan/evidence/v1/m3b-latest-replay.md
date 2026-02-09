# Evidence — v1 M3b (UserChoiceLatest capture/replay)

Environment: Windows 11, `HashVersion=1`

## Commands

- `cargo test`
- `cargo run -p fag-cli -- latest --ext .mp4`
- `cargo run -p fag-cli -- capture-latest --ext .mp4 --name potplayer`
- `cargo run -p fag-cli -- captures --ext .mp4`
- `cargo run -p fag-cli -- apply-latest --ext .mp4 --name potplayer`

## Notes

- `captures.json` 默认写入：`%APPDATA%\\FileAssocGuard\\captures.json`

