#!/usr/bin/env python3
"""Repair JVM std.fs integration fixtures before publishing the tranche."""

from pathlib import Path


def replace_once(source: str, old: str, new: str, label: str) -> str:
    if old not in source:
        raise SystemExit(f"{label} marker not found")
    return source.replace(old, new, 1)


def repair_native_file_delete() -> None:
    provider_path = Path("core/java/src/main/java/hara/truffle/HaraFileProvider.java")
    source = provider_path.read_text(encoding="utf-8")
    old = '''  String delete(String path, DeleteOptions options) throws IOException {
    String logical = HaraLogicalPath.normalise(path);
    if ("/".equals(logical)) throw failure("denied", "cannot delete the mounted root");
    Path host = scoped(logical);
    try {
      BasicFileAttributes attributes = attributes(host);
      if (attributes.isDirectory() && !attributes.isSymbolicLink()) {
        Files.delete(host);
      } else {
        Files.delete(host);
      }
      return logical;
    } catch (NoSuchFileException error) {
      if (options.missingOk()) return logical;
      throw map(error);
    } catch (Throwable error) {
      throw map(error);
    }
  }'''
    new = '''  String delete(String path, DeleteOptions options) throws IOException {
    String logical = HaraLogicalPath.normalise(path);
    if ("/".equals(logical)) throw failure("denied", "cannot delete the mounted root");
    Path host = scoped(logical);
    try {
      Files.delete(host);
      return logical;
    } catch (NoSuchFileException error) {
      if (options.missingOk()) return logical;
      throw map(error);
    } catch (Throwable error) {
      throw map(error);
    }
  }'''
    provider_path.write_text(
        replace_once(source, old, new, "HaraFileProvider missing-ok delete path"),
        encoding="utf-8",
    )


def repair_provider_test() -> None:
    test_path = Path("core/java/src/test/java/hara/truffle/HaraFileProviderTest.java")
    source = test_path.read_text(encoding="utf-8")
    marker = '''      assertFalse(provider.exists("/work/copied.bin"));
      assertTrue(provider.exists("/work/moved.bin"));

      String temporaryFile ='''
    replacement = '''      assertFalse(provider.exists("/work/copied.bin"));
      assertTrue(provider.exists("/work/moved.bin"));

      assertEquals(
          "/work/missing-ok",
          provider.delete("/work/missing-ok", new HaraFileProvider.DeleteOptions(true)));
      assertEquals(
          "not-found",
          assertThrows(
                  HaraFileProvider.Failure.class,
                  () ->
                      provider.delete(
                          "/work/missing", new HaraFileProvider.DeleteOptions(false)))
              .code());

      String temporaryFile ='''
    test_path.write_text(
        replace_once(source, marker, replacement, "HaraFileProvider missing-ok test"),
        encoding="utf-8",
    )


def repair_std_fs_test() -> None:
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


def main() -> None:
    repair_native_file_delete()
    repair_provider_test()
    repair_std_fs_test()


if __name__ == "__main__":
    main()
