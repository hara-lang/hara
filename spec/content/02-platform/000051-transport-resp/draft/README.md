# Hara RESP protocol

Status: **draft**
Contract version: **0.1.0**

The authoritative machine-readable contract is
[`transport-resp.edn`](transport-resp.edn). This README is its
human-readable companion.

The Hara native runtime can expose its evaluation broker over TCP using the
RESP wire protocol. A client connects, optionally negotiates a protocol
version with `HELLO`, and issues textual operations against an attached
session. The protocol lets editors, terminals, and other runtimes drive a
shared Hara process without embedding it.

The reference implementation is `rust/src/resp.rs` in the hara-lang
repository. The CLI routes `hara listen` (start a ROOT RESP listener) and
`hara connect` (attach a terminal to a remote listener) are declared in
`specs/02-platform/000001-cli/draft/hara-cli.edn`.

## Wire format

The wire encoding is RESP2. Every value is prefixed with a one-byte type
marker and terminated by CRLF (`\r\n`):

| Prefix | Type           | Encoding                                        |
|--------|----------------|-------------------------------------------------|
| `+`    | simple string  | `+<text>\r\n`                                   |
| `-`    | error          | `-<CODE> <message>\r\n`                         |
| `:`    | integer        | `:<signed 64-bit decimal>\r\n`                  |
| `$`    | bulk string    | `$<length>\r\n<bytes>\r\n`; `$-1\r\n` is null   |
| `*`    | array          | `*<count>\r\n` followed by count values; `*-1\r\n` is null |

Rules:

- Simple string, error, and integer payloads must be valid UTF-8 and must
  not contain CR or LF. Encoders reject line values containing either.
- Requests are arrays whose elements decode as text (simple strings, bulk
  strings, or integers). Any other request shape is answered with
  `-BAD_REQUEST`.
- Limits: a line is at most 64 KiB, a bulk payload at most 64 MiB, arrays
  nest at most 64 levels, and an array declares at most 65,536 elements.
  Exceeding a limit is a connection-level error.

## Connection lifecycle

```text
connect ──> optional HELLO (version negotiation)
        ──> operations (EVAL, COMPLETE, SESSION, COMMANDS, INFO)
        ──> QUIT ──> +OK and close
```

- Each connection carries an attached session name, initially `ROOT`, and a
  negotiated protocol version, initially `3`.
- `QUIT` replies `+OK` and closes the connection.
- The listener accepts connections concurrently; each connection is served
  independently but all sessions share the same broker, so a `def` made
  through one connection is visible to every other connection evaluating in
  the same session.

## HELLO

`HELLO [version]` negotiates the protocol dialect. Without an argument it
keeps the default version `3`. The reply is a flat array of key/value pairs:

```text
*8\r\n
$6\r\nSERVER\r\n  $4\r\nHARA\r\n
$8\r\nINSTANCE\r\n $<n>\r\nRUST-<pid>-<port>\r\n
$8\r\nPROTOCOL\r\n $1\r\n4\r\n
$4\r\nROOT\r\n     $<n>\r\n<working directory>\r\n
```

Versions below `4` select the legacy dialect; version `4` and above select
the streaming dialect described below.

## Operations

All operations are uppercase words in the first array element.

| Operation                | Arguments (legacy)      | Arguments (v4)         | Effect |
|--------------------------|-------------------------|------------------------|--------|
| `EVAL`                   | `<session> <source>`    | `<id> <source> [FILE <file> LINE <line> COLUMN <column>]` | Evaluate source; legacy targets an explicit session, v4 evaluates in the attached session |
| `COMPLETE`               | `<prefix>`              | `<id> <prefix>`        | Newline-joined completion candidates for the attached session |
| `SESSION NEW`            | `<name>`                | `<id> <name>`          | Create a session |
| `SESSION LIST`           | —                       | `<id>`                 | Newline-joined session names |
| `SESSION ATTACH`         | `<name>`                | `<id> <name>`          | Attach the connection to a session; fails with `NO_SESSION` when absent |
| `SESSION DETACH`         | —                       | `<id>`                 | Re-attach to `ROOT` |
| `SESSION INFO`           | —                       | `<id>`                 | Describe the attached session |
| `SESSION CLOSE`          | `<name>`                | `<id> <name>`          | Close a session |
| `COMMANDS`               | —                       | `<id>`                 | The operation vocabulary: `HELLO EVAL COMPLETE SESSION COMMANDS INFO QUIT` |
| `INFO`                   | —                       | `<id>`                 | Broker information for the attached session |
| `QUIT`                   | —                       | —                      | `+OK`, then close |

Unknown operations fail with `UNKNOWN_OP`.

## Legacy dialect (protocol 3 and below)

Each request produces exactly one reply:

- success: a single bulk string holding the displayed result;
- failure: a RESP error `-<CODE> <message>`.

In the legacy dialect only, `EVAL` takes the target session as its first
argument, so a client never has to `SESSION ATTACH` to evaluate elsewhere.

## Streaming dialect (protocol 4)

The second request element is a client-chosen request id. Each request
produces a two-reply envelope, allowing out-of-band matching of replies to
requests:

```text
success:  ["RESULT", <id>, <displayed value>]
          ["DONE",   <id>, "OK"]
failure:  ["ERROR",  <id>, <CODE>, <message>]
          ["DONE",   <id>, "ERROR"]
```

A missing id is reported as `?`.

### Evaluation diagnostics

For a protocol-4 `EVAL` failure, a server may append one fifth value to the
`ERROR` envelope. Older clients continue to use the first four fields; legacy
protocol-3 replies never include this value.

```text
["ERROR", <id>, "EVAL_ERROR", <message>,
 ["VERSION", 1,
  "MESSAGE", <message>,
  "EXCEPTION", ["MESSAGE", <message>, "CLASS", <class-or-null>,
                "CODE", <code-or-null>, "DATA", <bounded-readable-data>,
                "CAUSE", <nested-exception-or-null>, "THROWS", <locations>],
  "PRIMARY", ["FILE", <file-or-null>, "LINE", <line-or-null>, "COLUMN", <column-or-null>],
  "EXCERPT", ["START-LINE", <line>, "TEXT", <source-with-two-lines-of-context>],
  "FRAMES", [["FUNCTION", <function>, "NAMESPACE", <namespace-or-null>,
              "FILE", <file-or-null>, "LINE", <line-or-null>, "COLUMN", <column-or-null>], ...]]
```

`DATA` is a readable display limited to 16 KiB; diagnostics do not include
environment or process dumps. Editor clients should send `FILE`, `LINE`, and
`COLUMN` after the submitted source so the server can map the primary location
and source excerpt. Frames are ordered innermost first and should be rendered
as navigable locations when the editor can resolve the file or namespace.

## Error codes

| Code             | Meaning |
|------------------|---------|
| `BAD_REQUEST`    | Malformed request shape, missing argument, or unknown `SESSION` action |
| `EVAL_ERROR`     | Evaluation raised |
| `NO_SESSION`     | The named (or attached) session does not exist |
| `UNKNOWN_OP`     | Unknown operation |
| `INTERNAL_ERROR` | Broker failure unrelated to the request |

## Client library

`std.resp.client` is a separately required, blocking RESP2 client library
for speaking this protocol (or any RESP2 service) from Hara code. It exposes
`connect`, `call`, `write`, `read`, `pipeline`, `open?`, and `close`;
connections require the network capability. Bulk strings decode as UTF-8
strings by default and can be preserved as bytes with
`{:decode-bulk :bytes}`.

## Non-goals

- Authentication, TLS, and access control; the listener is intended for
  local development endpoints.
- RESP3 push types and out-of-band messages; the wire encoding stays RESP2
  even when the streaming dialect is negotiated.
- Cross-process namespace transactions beyond the shared broker semantics
  described above.

## Validation and parity

[`conformance/transport-resp.edn`](conformance/transport-resp.edn) defines
shared codec corpora and client transcripts for Truffle and Rust. Candidate
status requires identical normalized outcomes for RESP2 values, malformed
frames, resource limits, default legacy evaluation, protocol-4 envelopes,
errors, shared sessions, completion, command discovery, detach, and quit.

The current implementations are not yet transport-equivalent. The conformance
document records open differences in HELLO keys, protocol-3 interpretation,
array limits, detach and quit behavior, and completion/command result shapes.
These are validation failures to resolve, not fields that a parity harness may
silently normalize.
