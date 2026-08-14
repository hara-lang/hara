#!/usr/bin/env python3
"""Finalize generated #403 semantic-boundary sources before validation."""

from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    source = target.read_text()
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one replacement, found {count}")
    target.write_text(source.replace(old, new, 1))


replace_once(
    "core/rust/src/fiber/coroutine/observation.rs",
    "use super::semantic;\nuse super::super::*;\n",
    "use super::semantic;\nuse super::super::*;\nuse crate::kernel::{read_forms, SpannedForm};\n",
)

replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    "use super::semantic;\nuse super::super::*;\nuse crate::lang::data::{OrderedMap, Vector};\n",
    "use super::semantic;\nuse super::super::*;\nuse crate::kernel::{Position, Span, SpannedForm};\nuse crate::lang::data::{OrderedMap, Vector};\n",
)

replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''    let matches = source_forms
        .map(|forms| source_matches(forms, form))
        .unwrap_or_default();
    let unique = matches.len() == 1;
    let (path, span) = if unique {
        let matched = matches.into_iter().next().expect("one source match");
        (Some(matched.path), Some(span_snapshot(&matched.span)))
    } else {
        (None, None)
    };
    let source_candidates = if unique { 1 } else { matches.len() };
''',
    '''    let matches = source_forms
        .map(|forms| source_matches(forms, form))
        .unwrap_or_default();
    let source_candidates = matches.len();
    let unique = source_candidates == 1;
    let (path, span) = if unique {
        let matched = matches.into_iter().next().expect("one source match");
        (Some(matched.path), Some(span_snapshot(&matched.span)))
    } else {
        (None, None)
    };
''',
)

replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''        let returned = fiber.step_observed_snapshot("fixture/add.hal", limits);
        assert_eq!(returned.kind, EvalObservedBoundaryKind::Return);
        assert_eq!(returned.after.status, EvalObservationStatus::Returned);
''',
    '''        let mut returned = fiber.step_observed_snapshot("fixture/add.hal", limits);
        while returned.after.status == EvalObservationStatus::Paused {
            returned = fiber.step_observed_snapshot("fixture/add.hal", limits);
        }
        assert_eq!(returned.kind, EvalObservedBoundaryKind::Return);
        assert_eq!(returned.after.status, EvalObservationStatus::Returned);
''',
)
