# Hara federated tap protocol

A tap is an independently operated identity policy, release registry, and
immutable object distribution system. Clients pin its identity root
fingerprint locally and verify accepted Git records before trusting derived
services or downloaded bytes.

The official deployment exposes identity through `id.hara-lang.org`, package
and verification services through `packages.hara-lang.org`, and uncredentialed
media delivery through `assets.hara-lang.org`.

Git is the control-plane authority. There is no ledger or consensus layer.
