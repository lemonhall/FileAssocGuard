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

Supported workaround (new core behavior):

- Install `SetUserFTA.exe` (Christoph Kolbicz) and point `fag` to it:
  - set env `FAG_SETUSERFTA_EXE="C:\\path\\to\\SetUserFTA.exe"`; or
  - pass `--setuserfta "C:\\path\\to\\SetUserFTA.exe"`

Then rerun restore and verify via:

- `cargo run -p fag-cli -- restore --ext .mp4 --to vlc`
- `cargo run -p fag-cli -- read --ext .mp4`
