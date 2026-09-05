#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
import sys
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("connector_event_gateway.py")
SPEC = importlib.util.spec_from_file_location("connector_event_gateway", MODULE_PATH)
assert SPEC and SPEC.loader
module = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = module
SPEC.loader.exec_module(module)


class ConnectorCommandTest(unittest.TestCase):
    def test_ordinary_comment_is_ignored(self) -> None:
        self.assertIsNone(module.parse_command("please validate instrumentation"))
        self.assertIsNone(module.parse_command(" /hara-event instrumentation.java.validate"))

    def test_every_allow_listed_event_has_a_closed_route(self) -> None:
        expected = {
            "instrumentation.java.validate": "hara-instrumentation-java",
            "instrumentation.native.validate": "hara-instrumentation-native",
            "instrumentation.rust.validate": "hara-instrumentation-rust",
            "instrumentation.code-vm.validate": "hara-instrumentation-code-vm",
        }
        self.assertEqual(expected, {key: value["dispatch_type"] for key, value in module.EVENTS.items()})
        for event, dispatch_type in expected.items():
            request = module.parse_command(f"/hara-event {event}")
            self.assertIsNotNone(request)
            self.assertEqual(dispatch_type, request.dispatch_type)
            self.assertIsNone(request.requested_ref)

    def test_explicit_ref_is_validated(self) -> None:
        request = module.parse_command(
            "/hara-event instrumentation.java.validate ref=agent/942-truffle-events"
        )
        self.assertEqual("agent/942-truffle-events", request.requested_ref)
        sha = "A" * 40
        self.assertEqual(sha.lower(), module.validate_ref(sha))

    def test_unknown_or_extended_commands_are_rejected(self) -> None:
        bad = [
            "/hara-event unknown.event",
            "/hara-event instrumentation.java.validate extra=value",
            "/hara-event  instrumentation.java.validate",
            "/hara-event instrumentation.java.validate ref=main extra=true",
            "/hara-event instrumentation.java.validate\nsecond-line",
        ]
        for body in bad:
            with self.subTest(body=body):
                with self.assertRaises(module.CommandError):
                    module.parse_command(body)

    def test_unsafe_refs_are_rejected(self) -> None:
        bad = [
            "HEAD",
            "refs/heads/main",
            "../main",
            "feature..next",
            "feature@{one}",
            "feature//next",
            "feature\\next",
            "/main",
            "main/",
            ".hidden",
            "feature/.hidden",
            "feature.lock",
            "feature.",
            "feature name",
            "feature:next",
            "feature~1",
        ]
        for ref in bad:
            with self.subTest(ref=ref):
                with self.assertRaises(module.CommandError):
                    module.validate_ref(ref)

    def test_permission_matrix_is_closed(self) -> None:
        for permission in ("write", "maintain", "admin", "WRITE"):
            self.assertTrue(module.permission_allows(permission))
        for permission in (None, "", "read", "triage", "none"):
            self.assertFalse(module.permission_allows(permission))


class ConnectorEventTest(unittest.TestCase):
    def issue_payload(self, body: str) -> dict:
        return {
            "issue": {"number": 958, "html_url": "https://github.test/issues/958"},
            "comment": {
                "id": 12345,
                "body": body,
                "html_url": "https://github.test/issues/958#issuecomment-12345",
                "user": {"login": "trusted-user"},
                "performed_via_github_app": {"slug": "chatgpt-codex-connector"},
            },
            "sender": {"login": "trusted-user"},
        }

    def test_issue_comment_preserves_immutable_source_evidence(self) -> None:
        request = module.request_from_issue_comment(
            self.issue_payload("/hara-event instrumentation.native.validate ref=main")
        )
        self.assertEqual("comment-12345", request.request_id)
        self.assertEqual(958, request.issue_number)
        self.assertEqual(12345, request.comment_id)
        self.assertEqual("chatgpt-codex-connector", request.source_app)
        self.assertEqual("main", request.command.requested_ref)

    def test_workflow_dispatch_is_versioned_and_bounded(self) -> None:
        payload = {
            "inputs": {
                "event": "instrumentation.rust.validate",
                "ref": "main",
                "issue_number": "958",
            },
            "sender": {"login": "trusted-user"},
        }
        request = module.request_from_workflow_dispatch(payload, 444, 2)
        client = module.build_client_payload(
            request,
            resolved_ref="main",
            resolved_actor="trusted-user",
            gateway_run_id=444,
        )
        self.assertEqual(module.SCHEMA, client["schema"])
        self.assertEqual("manual-444", client["request_id"])
        self.assertEqual("instrumentation.rust.validate", client["event"])
        self.assertLessEqual(len(client), 10)
        json.dumps(client)

    def test_cli_reports_malformed_command_without_evaluation(self) -> None:
        payload = self.issue_payload("/hara-event instrumentation.java.validate ref=../main")
        with tempfile.TemporaryDirectory() as directory:
            event_path = Path(directory, "event.json")
            output_path = Path(directory, "output.txt")
            event_path.write_text(json.dumps(payload), encoding="utf-8")
            result = module.cli(
                [
                    "--event-name",
                    "issue_comment",
                    "--event-path",
                    str(event_path),
                    "--output",
                    str(output_path),
                    "--run-id",
                    "10",
                    "--run-attempt",
                    "1",
                ]
            )
            self.assertEqual(0, result)
            output = output_path.read_text(encoding="utf-8")
            self.assertIn("command=true", output)
            self.assertIn("valid=false", output)
            self.assertIn("actor=trusted-user", output)


if __name__ == "__main__":
    unittest.main()