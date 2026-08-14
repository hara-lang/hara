from pathlib import Path

path = Path("core/java/src/test/java/hara/truffle/MainTest.java")
text = path.read_text()
old = '''      assertEquals(
          0,
          Main.run(
              new String[] {
                "--project",
                root.toString(),
                "test",
                root.resolve("test/demo_app/main_test.hal").toString()
              },
              stdout,
              stderr));'''
new = '''      int explicitTestStatus =
          Main.run(
              new String[] {
                "--project",
                root.toString(),
                "test",
                root.resolve("test/demo_app/main_test.hal").toString()
              },
              stdout,
              stderr);
      assertEquals(error.toString(StandardCharsets.UTF_8), 0, explicitTestStatus);'''
count = text.count(old)
if count != 1:
    raise SystemExit(f"expected one explicit project test assertion, found {count}")
path.write_text(text.replace(old, new, 1))
