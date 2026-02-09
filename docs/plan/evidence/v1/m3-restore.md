# v1 M3 Evidence — restore (registry write)

## PRD Trace

- `REQ-014`（restore 部分）

## Candidates (mp4)

Run: `cargo run -p fag-cli -- progids --ext .mp4`  
Example output:

```json
{"ext":".mp4","progids":["PotPlayerMini64.mp4","QQLive.mp4","VLC.mp4"]}
```

## Restore Attempt (this machine)

Run: `cargo run -p fag-cli -- restore --ext .mp4 --to vlc`  
Result: Legacy restore fails on this machine because `HashVersion=1` (UserChoiceLatest enabled).

Workaround (temporary):

- Disable UserChoiceLatest feature flags (ViveTool) and reboot:
  - `vivetool /disable /id:43229420`
  - `vivetool /disable /id:27623730`

After reboot, rerun restore and verify via:

- `cargo run -p fag-cli -- restore --ext .mp4 --to vlc`
- `cargo run -p fag-cli -- read --ext .mp4`
