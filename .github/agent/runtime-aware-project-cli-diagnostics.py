from pathlib import Path

path = Path("core/java/src/test/java/hara/truffle/MainTest.java")
text = path.read_text()

old_project = '''      assertEquals(0, Main.run(new String[] {"--project", root.toString(), "test"}, stdout, stderr));'''
new_project = '''      int projectTestStatus =
          Main.run(new String[] {"--project", root.toString(), "test"}, stdout, stderr);
      assertEquals(
          output.toString(StandardCharsets.UTF_8)
              + "\\nerror:\\n"
              + error.toString(StandardCharsets.UTF_8),
          0,
          projectTestStatus);'''
count = text.count(old_project)
if count != 1:
    raise SystemExit(f"expected one project-wide test assertion, found {count}")
text = text.replace(old_project, new_project, 1)

old_explicit = '''      assertEquals(
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
new_explicit = '''      int explicitTestStatus =
          Main.run(
              new String[] {
                "--project",
                root.toString(),
                "test",
                root.resolve("test/demo_app/main_test.hal").toString()
              },
              stdout,
              stderr);
      assertEquals(
          output.toString(StandardCharsets.UTF_8)
              + "\\nerror:\\n"
              + error.toString(StandardCharsets.UTF_8),
          0,
          explicitTestStatus);'''
count = text.count(old_explicit)
if count != 1:
    raise SystemExit(f"expected one explicit project test assertion, found {count}")
path.write_text(text.replace(old_explicit, new_explicit, 1))
