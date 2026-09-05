# `hara:values`

`hara:values@0.1.0` is the portable Component Model contract for Hara's
immutable value space. Components import `hara:values/values@0.1.0.{value}`
and declare their Hara argument or result as `:value`.

The value is an explicit post-order graph: all child indices are earlier than
their parent, `root` names the final node, and every node must be reachable
from it. The form retains collection category, metadata, ordered iteration,
map-entry identity, and Hara equality/ordering semantics; it is not JSON or an
opaque HTA byte frame.

The boundary rejects mutable, lazy, callable, resource, provider, process, and
session-owned values. It also rejects non-finite floats. See the extension
contract and native-host conformance tests for the exact error surface.
