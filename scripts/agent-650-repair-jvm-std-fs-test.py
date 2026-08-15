#!/usr/bin/env python3
"""Repair JVM std.fs integration fixtures before publishing the tranche."""

from pathlib import Path


def replace_once(source: str, old: str, new: str, label: str) -> str:
    if old not in source:
        raise SystemExit(f"{label} marker not found")
    return source.replace(old, new, 1)


def main() -> None:
    test_path = Path("core/java/src/test/java/hara/truffle/StdFsTest.java")
    source = test_path.read_text(encoding="utf-8")
    source = replace_once(
        source,
        '          "\\\"/simple-delete\\\"",',
        '          "/simple-delete",',
        "StdFsTest scalar delete expectation",
    )
    source = replace_once(
        source,
        '          "\\\"/missing-ok\\\"",',
        '          "/missing-ok",',
        "StdFsTest missing-ok expectation",
    )
    source = replace_once(
        source,
        '+ "                  (deref (std.fs.walk/walk \\\"/\\\"))))))")',
        '+ "                  (deref (std.fs.walk/walk \\\"/\\\")))))")',
        "StdFsTest walk expression delimiter",
    )

    missing_ok_marker = '''      assertEquals(
          "/missing-ok",
          context
              .eval(
                  HaraLanguage.ID,
                  "(deref (std.fs/delete \\\"/missing-ok\\\" {:missing-ok? true}))")
              .toString());'''
    missing_ok_contract = '''      assertEquals(
          "[\\\"/native-missing-ok\\\" true true]",
          context
              .eval(
                  HaraLanguage.ID,
                  "(do (require 'std.fs)"
                      + "    [(deref (File/delete \\\"/native-missing-ok\\\""
                      + "                         {:missing-ok? true}))"
                      + "     (:missing-ok?"
                      + "      (merge std.fs/delete-default-options"
                      + "             {:missing-ok? true}))"
                      + "     (:missing-ok? {:missing-ok? true})])")
              .toString());

''' + missing_ok_marker
    source = replace_once(
        source,
        missing_ok_marker,
        missing_ok_contract,
        "StdFsTest missing-ok option contract",
    )
    test_path.write_text(source, encoding="utf-8")


if __name__ == "__main__":
    main()
