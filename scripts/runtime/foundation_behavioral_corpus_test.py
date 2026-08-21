import copy
import json
import sys
import unittest
from pathlib import Path

import foundation_behavioral_corpus as corpus


ROOT = Path(__file__).resolve().parents[2]


def observation_command(observation, exit_code=0):
    payload = json.dumps(observation)
    return [
        sys.executable,
        "-c",
        f"import sys; print({payload!r}); sys.exit({exit_code})",
    ]


def case(reference=None, hara=None, status="portable"):
    reference = reference or {"outcome": "success", "value": 3, "type": "integer", "display": "3"}
    hara = hara or reference
    return {
        "id": "test/example",
        "status": status,
        "disposition_reason": "not applicable",
        "provenance": {
            "foundation": {
                "path": "foundation.diff",
                "sha256": "a" * 64,
                "source": {"path": "src/example.clj", "sha256": "b" * 64},
                "test": {"path": "test/example_test.clj", "sha256": "c" * 64},
            },
            "hara": {
                "path": "hara.diff",
                "sha256": "d" * 64,
                "source": {"path": "core/lib/src/example.hal", "sha256": "e" * 64},
                "test": {"path": "core/lib/test/example_test.hal", "sha256": "f" * 64},
            },
        },
        "coordinates": {
            "namespace": "example",
            "symbol": "value",
            "grammar": "hara/v1",
            "runtime": "test",
        },
        "input": {"form": "(value)"},
        "expectation": {"outcome": "success", "value": 3},
        "observations": {
            "lifecycle": None,
            "namespace": None,
            "mutation": None,
            "state": None,
        },
        "requirements": {"deterministic": True, "ordering": "none"},
        "commands": {
            "reference": observation_command(reference),
            "hara": observation_command(hara),
        },
    }


def test_corpus(cases):
    return {
        "document/type": "foundation-behavioral-corpus",
        "document/version": 1,
        "references": {
            "foundation": {
                "repository": "foundation",
                "revision": "a" * 40,
            }
        },
        "normalization": [
            {"id": "path", "description": "paths"},
            {"id": "process-wrapper", "description": "wrappers"},
            {"id": "generated-identity", "description": "identities"},
        ],
        "cases": cases,
    }


class FoundationBehavioralCorpusTest(unittest.TestCase):
    def test_committed_corpus_is_versioned_and_valid(self):
        loaded = corpus.load_corpus(ROOT / "core/spec/code-migrate/foundation-behavioral-corpus.json")
        self.assertEqual(1, loaded["document/version"])
        self.assertEqual(2, len(loaded["cases"]))

    def test_normalization_is_named_and_recursive(self):
        value = {
            "diagnostics": "/tmp/work/550e8400-e29b-41d4-a716-446655440000",
            "nested": ["Process[pid=1]: generated-worker"],
        }
        actual = corpus.normalize(value, {"path", "process-wrapper", "generated-identity"})
        self.assertEqual("<path>", actual["diagnostics"])
        self.assertEqual(["<generated>"], actual["nested"])

    def test_differential_report_is_deterministic_and_non_vacuous(self):
        source = {"outcome": "success", "value": 3, "type": "integer", "display": "3"}
        valid = test_corpus([case(source, source)])
        first = corpus.run_corpus(valid)
        second = corpus.run_corpus(valid)
        self.assertTrue(first["conformant"])
        self.assertEqual(corpus.render_report(first), corpus.render_report(second))

        incorrect = test_corpus([case(source, {"outcome": "success", "value": 4})])
        report = corpus.run_corpus(incorrect)
        self.assertFalse(report["conformant"])
        self.assertEqual("test/example", report["results"][0]["case_id"])
        self.assertIn("value", [item["field"] for item in report["results"][0]["differences"]])

    def test_deferred_case_is_explicitly_skipped(self):
        deferred = case(status="deferred")
        report = corpus.run_corpus(test_corpus([deferred]))
        self.assertTrue(report["conformant"])
        self.assertEqual(1, report["skipped"])
        self.assertTrue(report["results"][0]["skipped"])

    def test_expected_failure_is_compared(self):
        failure = case(
            {"outcome": "failure", "diagnostics": "bad input"},
            {"outcome": "failure", "diagnostics": "bad input"},
        )
        failure["expectation"] = {"outcome": "failure"}
        failure["commands"]["reference"] = observation_command({}, 1)
        failure["commands"]["hara"] = observation_command({}, 1)
        report = corpus.run_corpus(test_corpus([failure]))
        self.assertTrue(report["conformant"])


if __name__ == "__main__":
    unittest.main()
