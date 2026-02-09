# v1 M1 Evidence — registry read

## PRD Trace

- `REQ-010`

## Tests

Run: `cargo test -p fag-core`  
Result: PASS (6 tests)

## CLI

Run: `cargo run -p fag-cli -- read --ext .mp4`  
Example output:

```json
{"ext":".mp4","status":"NOT_SET","prog_id":null,"hash":null,"last_write_time_filetime":null}
```

