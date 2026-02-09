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
Result: FAIL on this machine because `HashVersion=1` (UserChoiceLatest enabled).

Expected workaround:

- `vivetool /disable /id:43229420`
- `vivetool /disable /id:27623730`
- reboot

After disabling, rerun the restore command and verify via:

- `cargo run -p fag-cli -- read --ext .mp4`
