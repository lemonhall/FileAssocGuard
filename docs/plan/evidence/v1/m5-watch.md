# Evidence — v1 M5 (watch + logs)

## Commands

- `cargo run -p fag-cli -- watch --ext .mp4 --name vlc --interval 5`
- `cargo run -p fag-cli -- watch-rules --interval 5`

## Expected

- When tampered, CLI prints `status=TAMPERED` then `status=APPLIED`
- Log file appends JSON lines:
  - `%APPDATA%\\FileAssocGuard\\guard.log`

