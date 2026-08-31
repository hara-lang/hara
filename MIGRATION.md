# Archive migration ledger

## Tranche 1 — portable codecs

The first archive migration tranche ports the portable, Foundation-only
encoding libraries from `hara-archive-v1` at
`df48836e9d9e10b34e4481b13194ec29f4068515`.

| Archive namespace | Canonical Hara namespace | Status |
| --- | --- | --- |
| `std.lib.encode.hex` | `std.lib.encode.hex` | source and behavioral tests migrated |
| `std.lib.encode.base64` | `std.lib.encode.base64` | source and behavioral tests migrated |
| `std.lib.encode.url` | `std.lib.encode.url` | source and behavioral tests migrated |
| `std.lib.encode.form` | `std.lib.encode.form` | source and behavioral tests migrated |

Each codec remains a single source namespace. There is no `:internal` /
`:facade` split: that pattern is reserved for public surfaces composed from
multiple implementation files. `encode` and `decode` are the recommended
public Vars; every helper has a path-matched direct `Test/check` assertion.

The migrated tests cover canonical output, malformed input with structured
error data, UTF-8 boundaries, repeated form pairs, and all-byte round trips
where applicable. The source package is verified with Hara Native before a
later tranche is accepted.
