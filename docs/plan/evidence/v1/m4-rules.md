# Evidence — v1 M4 (rules.json + check)

## Commands

- `cargo test`
- `cargo run -p fag-cli -- rules list`
- `cargo run -p fag-cli -- rules add --ext .mp4 --name vlc`
- `cargo run -p fag-cli -- check` (verify `$LASTEXITCODE`)
- `cargo run -p fag-cli -- watch-rules --interval 5`

## Notes

- `rules.json` default: `%APPDATA%\\FileAssocGuard\\rules.json`

