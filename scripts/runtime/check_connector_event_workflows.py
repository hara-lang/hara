#!/usr/bin/env python3
"""Static contract checks for connector event workflows."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GATEWAY = ROOT / ".github/workflows/connector-instrumentation-events.yml"
ROUTES = {
    ROOT / ".github/workflows/instrumentation-java.yml": ("hara-instrumentation-java", "contracts"),
    ROOT / ".github/workflows/instrumentation-native.yml": ("hara-instrumentation-native", "focused"),
    ROOT / ".github/workflows/instrumentation-rust.yml": ("hara-instrumentation-rust", "focused"),
    ROOT / ".github/workflows/instrumentation-code-vm.yml": ("hara-instrumentation-code-vm", "focused"),
}


def require(text: str, needle: str, file: Path) -> None:
    if needle not in text:
        raise SystemExit(f"{file}: missing required connector contract: {needle}")


def reject(text: str, needle: str, file: Path) -> None:
    if needle in text:
        raise SystemExit(f"{file}: forbidden connector contract text: {needle}")


def main() -> int:
    gateway = GATEWAY.read_text(encoding="utf-8")
    for required in (
        "issue_comment:",
        "types: [created]",
        "workflow_dispatch:",
        "contents: write",
        "issues: write",
        "pull-requests: read",
        "scripts/runtime/connector_event_gateway.py",
        "createDispatchEvent",
        "getCollaboratorPermissionLevel",
        "hara-connector-request:",
    ):
        require(gateway, required, GATEWAY)
    for forbidden in ("eval ", "bash -c", "sh -c", "${{ github.event.comment.body }}"):
        reject(gateway, forbidden, GATEWAY)

    for file, (dispatch_type, job_id) in ROUTES.items():
        text = file.read_text(encoding="utf-8")
        require(text, "repository_dispatch:", file)
        require(text, dispatch_type, file)
        require(text, "github.event.client_payload.ref || github.ref", file)
        require(text, "github.event.client_payload.request_id", file)
        require(text, f"needs.{job_id}.result", file)
        require(text, "issues: write", file)
        require(text, "hara-connector-completion:", file)

    print("connector instrumentation workflow contracts are consistent")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())