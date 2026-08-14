#!/usr/bin/env python3
"""Finalize generated explicit semantic event sources before validation."""

from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    source = target.read_text()
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one replacement, found {count}")
    target.write_text(source.replace(old, new, 1))


replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''            .find(|semantic| semantic.focus.form == "x" && semantic.result.display == "41")
''',
    '''            .find(|semantic| {
                semantic.focus.form == "x"
                    && semantic
                        .result
                        .as_ref()
                        .is_some_and(|result| result.display == "41")
            })
''',
)

replace_once(
    "core/rust/src/fiber.rs",
    '''                let mut effect_target = name.clone();
''',
    '''                let effect_target;
''',
)

replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''        assert_eq!(first.kind, EvalObservedBoundaryKind::Continue);
''',
    '''        assert_eq!(first.kind, EvalObservedBoundaryKind::Semantic);
''',
)

replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''            .find(|semantic| semantic.focus.form == "(* 2 3)")
            .expect("inner multiply boundary");
''',
    '''            .find(|semantic| {
                semantic.focus.form == "(* 2 3)"
                    && semantic
                        .result
                        .as_ref()
                        .is_some_and(|result| result.display == "6")
            })
            .expect("inner multiply return boundary");
''',
)

replace_once(
    "core/rust/src/fiber/coroutine/snapshot.rs",
    '''                semantic.rule == "form/return"
                    && semantic.focus.form == "(* 2 3)"
''',
    '''                semantic.rule == "value/return"
                    && semantic.focus.form == "(* 2 3)"
''',
)
