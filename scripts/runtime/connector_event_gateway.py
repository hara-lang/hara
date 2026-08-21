#!/usr/bin/env python3
"""Parse and route connector-authored instrumentation workflow requests.

The module is deliberately pure except for the small CLI adapter at the end.
GitHub API authorization, ref resolution, dispatch, and commenting stay in the
workflow.  No comment content is ever evaluated as shell or Python source.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping

SCHEMA = "hara.connector.event/0-alpha"
COMMAND = "/hara-event"
AUTHORIZED_PERMISSIONS = frozenset({"admin", "maintain", "write"})

EVENTS: dict[str, dict[str, str]] = {
    "instrumentation.java.validate": {
        "dispatch_type": "hara-instrumentation-java",
        "workflow": ".github/workflows/instrumentation-java.yml",
    },
    "instrumentation.native.validate": {
        "dispatch_type": "hara-instrumentation-native",
        "workflow": ".github/workflows/instrumentation-native.yml",
    },
    "instrumentation.rust.validate": {
        "dispatch_type": "hara-instrumentation-rust",
        "workflow": ".github/workflows/instrumentation-rust.yml",
    },
    "instrumentation.code-vm.validate": {
        "dispatch_type": "hara-instrumentation-code-vm",
        "workflow": ".github/workflows/instrumentation-code-vm.yml",
    },
}

_SAFE_REF = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._/-]{0,199}$")
_FULL_SHA = re.compile(r"^[0-9a-fA-F]{40}$")


class CommandError(ValueError):
    """A connector command was recognized but rejected."""


@dataclass(frozen=True)
class CommandRequest:
    event: str
    dispatch_type: str
    workflow: str
    requested_ref: str | None


@dataclass(frozen=True)
class EventRequest:
    command: CommandRequest
    request_id: str
    issue_number: int
    comment_id: int
    actor: str
    source_app: str
    source_url: str


def permission_allows(permission: str | None) -> bool:
    return (permission or "").lower() in AUTHORIZED_PERMISSIONS


def validate_ref(value: str) -> str:
    """Validate the restricted same-repository ref grammar accepted by the gateway."""

    if not value:
        raise CommandError("ref must be non-empty")
    if _FULL_SHA.fullmatch(value):
        return value.lower()
    if not _SAFE_REF.fullmatch(value):
        raise CommandError("ref contains unsupported characters or exceeds 200 characters")
    if value in {"HEAD", ".", ".."}:
        raise CommandError("ref is reserved")
    if value.startswith("refs/") or value.startswith("/") or value.endswith("/"):
        raise CommandError("ref must be a branch, tag, or full commit SHA without a refs/ prefix")
    if ".." in value or "@{" in value or "//" in value or "\\" in value:
        raise CommandError("ref contains a forbidden Git ref sequence")
    if value.endswith("."):
        raise CommandError("ref cannot end with a dot")
    for component in value.split("/"):
        if not component or component.startswith(".") or component.endswith(".lock"):
            raise CommandError("ref contains a forbidden Git ref component")
    return value


def _single_line(body: str) -> str:
    text = body.rstrip("\r\n")
    if "\n" in text or "\r" in text:
        raise CommandError("command must be a single line")
    if text != text.strip():
        raise CommandError("command cannot have leading or trailing whitespace")
    return text


def parse_command(body: Any) -> CommandRequest | None:
    """Parse an exact connector command, returning None for ordinary comments."""

    if not isinstance(body, str):
        return None
    candidate = body.rstrip("\r\n")
    if not candidate.startswith(COMMAND):
        return None
    text = _single_line(body)
    tokens = text.split(" ")
    if any(token == "" for token in tokens):
        raise CommandError("command must use single spaces")
    if len(tokens) not in {2, 3} or tokens[0] != COMMAND:
        raise CommandError("expected /hara-event <event> [ref=<git-ref>]")

    event = tokens[1]
    route = EVENTS.get(event)
    if route is None:
        raise CommandError("event is not allow-listed")

    requested_ref: str | None = None
    if len(tokens) == 3:
        option = tokens[2]
        if not option.startswith("ref=") or option.count("=") != 1:
            raise CommandError("the only optional argument is ref=<git-ref>")
        requested_ref = validate_ref(option[4:])

    return CommandRequest(
        event=event,
        dispatch_type=route["dispatch_type"],
        workflow=route["workflow"],
        requested_ref=requested_ref,
    )


def _positive_int(value: Any, field: str, *, allow_zero: bool = False) -> int:
    try:
        parsed = int(value)
    except (TypeError, ValueError) as error:
        raise CommandError(f"{field} must be an integer") from error
    minimum = 0 if allow_zero else 1
    if parsed < minimum:
        raise CommandError(f"{field} must be at least {minimum}")
    return parsed


def request_from_issue_comment(payload: Mapping[str, Any]) -> EventRequest | None:
    comment = payload.get("comment") or {}
    issue = payload.get("issue") or {}
    command = parse_command(comment.get("body"))
    if command is None:
        return None
    comment_id = _positive_int(comment.get("id"), "comment id")
    issue_number = _positive_int(issue.get("number"), "issue number")
    user = comment.get("user") or {}
    performed = comment.get("performed_via_github_app") or {}
    actor = str(user.get("login") or payload.get("sender", {}).get("login") or "")
    if not actor:
        raise CommandError("comment actor is missing")
    return EventRequest(
        command=command,
        request_id=f"comment-{comment_id}",
        issue_number=issue_number,
        comment_id=comment_id,
        actor=actor,
        source_app=str(performed.get("slug") or "direct"),
        source_url=str(comment.get("html_url") or issue.get("html_url") or ""),
    )


def request_from_workflow_dispatch(
    payload: Mapping[str, Any], run_id: int, run_attempt: int
) -> EventRequest:
    inputs = payload.get("inputs") or {}
    event = str(inputs.get("event") or "")
    ref = validate_ref(str(inputs.get("ref") or "main"))
    route = EVENTS.get(event)
    if route is None:
        raise CommandError("event is not allow-listed")
    issue_text = str(inputs.get("issue_number") or "0")
    issue_number = _positive_int(issue_text, "issue number", allow_zero=True)
    actor = str((payload.get("sender") or {}).get("login") or "")
    if not actor:
        raise CommandError("workflow actor is missing")
    return EventRequest(
        command=CommandRequest(event, route["dispatch_type"], route["workflow"], ref),
        request_id=f"manual-{run_id}",
        issue_number=issue_number,
        comment_id=0,
        actor=actor,
        source_app="workflow-dispatch",
        source_url="",
    )


def build_client_payload(
    request: EventRequest, *, resolved_ref: str, resolved_actor: str, gateway_run_id: int
) -> dict[str, Any]:
    """Build the bounded repository_dispatch payload (GitHub permits 10 top-level keys)."""

    ref = validate_ref(resolved_ref)
    if not resolved_actor:
        raise CommandError("resolved actor is missing")
    payload: dict[str, Any] = {
        "schema": SCHEMA,
        "request_id": request.request_id,
        "event": request.command.event,
        "issue_number": request.issue_number,
        "comment_id": request.comment_id,
        "actor": resolved_actor,
        "ref": ref,
        "source_app": request.source_app,
        "source_url": request.source_url,
        "gateway_run_id": gateway_run_id,
    }
    if len(payload) > 10:
        raise AssertionError("repository_dispatch client payload exceeds GitHub's key limit")
    return payload


def parse_event(
    event_name: str, payload: Mapping[str, Any], run_id: int, run_attempt: int
) -> EventRequest | None:
    if event_name == "issue_comment":
        return request_from_issue_comment(payload)
    if event_name == "workflow_dispatch":
        return request_from_workflow_dispatch(payload, run_id, run_attempt)
    raise CommandError(f"unsupported gateway event: {event_name}")


def _write_output(path: Path, values: Mapping[str, Any]) -> None:
    with path.open("a", encoding="utf-8") as stream:
        for key, value in values.items():
            text = str(value)
            if "\n" in text or "\r" in text:
                raise ValueError(f"output {key} is not single-line")
            stream.write(f"{key}={text}\n")


def _event_actor(payload: Mapping[str, Any]) -> str:
    comment = payload.get("comment") or {}
    user = comment.get("user") or {}
    sender = payload.get("sender") or {}
    return str(user.get("login") or sender.get("login") or "")


def cli(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--event-name", required=True)
    parser.add_argument("--event-path", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--run-id", type=int, required=True)
    parser.add_argument("--run-attempt", type=int, required=True)
    args = parser.parse_args(argv)

    try:
        payload = json.loads(args.event_path.read_text(encoding="utf-8"))
        request = parse_event(args.event_name, payload, args.run_id, args.run_attempt)
    except (CommandError, json.JSONDecodeError, OSError) as error:
        # A recognized malformed command is reported by the workflow after it checks actor authority.
        _write_output(
            args.output,
            {
                "command": "true",
                "valid": "false",
                "error": str(error),
                "actor": _event_actor(payload if "payload" in locals() else {}),
            },
        )
        return 0

    if request is None:
        _write_output(args.output, {"command": "false", "valid": "true"})
        return 0

    command = request.command
    _write_output(
        args.output,
        {
            "command": "true",
            "valid": "true",
            "schema": SCHEMA,
            "event": command.event,
            "dispatch_type": command.dispatch_type,
            "workflow": command.workflow,
            "requested_ref": command.requested_ref or "",
            "request_id": request.request_id,
            "issue_number": request.issue_number,
            "comment_id": request.comment_id,
            "actor": request.actor,
            "source_app": request.source_app,
            "source_url": request.source_url,
        },
    )
    return 0


if __name__ == "__main__":
    sys.exit(cli())