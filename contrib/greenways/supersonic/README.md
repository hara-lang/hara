# Greenways Supersonic

Supersonic defines a canonical, declarative audio graph and a Hara/JavaScript
adapter for exchanging that graph with local providers.

The contribution owns:

- the graph and control specification;
- graph normalization and typed update semantics;
- the `gw.audio.supersonic` HAL boundary; and
- browser provider lifecycle tests.

It does not own the browser audio engine itself. Playback resources remain
page-owned and require explicit project capability plus a user gesture.

## Package Showcases

`showcase.edn` publishes two complete source-local stories:

- **Inspect an audio graph** renders the portable value without requesting audio;
- **Play and reshape Glass Signal** requests `:audio/playback`, starts silently,
  and exposes user-authorized live controls.

Both stories are ordinary Hara projects with named EDN states and explicit
Workspace surfaces, ready for commit-pinned publication in the Hara Package
Gallery.
