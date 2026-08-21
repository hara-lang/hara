# `memory.v1` Wasm binding contract

`memory.v1` is a synchronous, import-free ABI for Wasm modules that expose one
linear memory. The module must export the memory named by the interface and may
export `alloc(i32) -> i32` and `free(i32) -> void`. `reallocate` is reserved
until a later ABI revision and is rejected by this implementation.

Scalar Hara values use their direct Wasm representation. `string` and `bytes`
arguments lower to `(i32 pointer, i32 length)`; zero-length values use
`(0, 0)`. `string` and `bytes` results use an `i64` whose low 32 bits are the
pointer and high 32 bits are the byte length. Results are copied before any
release operation, and string results must be valid UTF-8.

## Ownership

| value | ownership | host action |
| --- | --- | --- |
| input | `borrowed` | copy for the call; never call `free` |
| input | `transferred` | call `free` only if invocation does not complete |
| result | `caller` | copy, then call `free` exactly once |
| result | `callee` | copy; never call `free` |

The host deduplicates cleanup pointers and attempts every required release.
When invocation and cleanup both fail, the reported error retains both
classifications.

## Limits and errors

Hosts bound linear memory to 64 MiB, each value to 16 MiB, total input copies
to 32 MiB, and aggregate input/output copies to 48 MiB. They reject imports,
start functions, missing or malformed memory exports, invalid signatures,
negative or overflowing ranges, allocator failures, traps, invalid UTF-8,
oversized results, and release failures before exposing a binding.

Packages record SHA-256 digests for the module, canonical interface, canonical
binding plan, and build product inputs. Runtime installation verifies those
digests and does not execute a module during static inspection.
