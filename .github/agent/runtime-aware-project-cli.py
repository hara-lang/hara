from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected one match in {path}, found {count}: {old[:120]!r}")
    file.write_text(text.replace(old, new, 1))


main = "core/java/src/main/java/hara/truffle/Main.java"
replace_once(
    main,
    '''            + " :project/main " + namespace + ".main\\n :project/capabilities #{}\\n :project/dependencies {}\\n"
            + " :jvm/dependencies []\\n :jvm/source-paths [\\"src-java\\"]\\n :jvm/target-path \\"target/classes\\"}\\n");''',
    '''            + " :project/main " + namespace + ".main\\n :project/capabilities #{}\\n :project/dependencies {}\\n"
            + " :project/runtime-profiles\\n"
            + " {:jvm {:runtime/native-source-paths [\\"src-java\\"]\\n"
            + "        :runtime/target-path \\"target/jvm/classes\\"\\n"
            + "        :runtime/dependencies {:maven {}}}}}\\n");''',
)

replace_once(
    main,
    '''    String manifest = Files.readString(project.descriptor());
    int dependencyKey = manifest.indexOf(":project/dependencies");
    int dependencyOpen = dependencyKey < 0 ? -1 : manifest.indexOf('{', dependencyKey);
    int dependencyClose = dependencyOpen < 0 ? -1 : matchingBrace(manifest, dependencyOpen);
    if (dependencyOpen < 0 || dependencyClose < 0)
      throw new HaraException("project.edn :project/dependencies must be an EDN map");
    String dependencies = manifest.substring(dependencyOpen + 1, dependencyClose).trim();
    if (!dependencies.isEmpty()) {
      java.util.regex.Matcher coordinates =
          java.util.regex.Pattern.compile("\\\\\\\"[^\\\\\\\"]+\\\\\\\"\\\\s*\\\\{").matcher(dependencies);
      int count = 0;
      while (coordinates.find()) count++;
      throw new HaraException("project sync requires the reviewed registry client to resolve " + count + " declared dependencies");
    }
''',
    '''    if (!project.haraDependencies().isEmpty()) {
      throw new HaraException(
          "project sync requires the reviewed registry client to resolve "
              + project.haraDependencies().size()
              + " active Hara dependencies");
    }
''',
)

test = "core/java/src/test/java/hara/truffle/MainTest.java"
replace_once(
    test,
    '''              + ":project/dependencies {} "
              + ":jvm/dependencies [[org.apache.commons/commons-lang3 \\"3.12.0\\"]] "
              + ":jvm/source-paths [\\"src-java\\"] :jvm/target-path \\"target/classes\\"}");''',
    '''              + ":project/dependencies {} "
              + ":project/runtime-profiles {:jvm {"
              + ":runtime/native-source-paths [\\"src-java\\"] "
              + ":runtime/target-path \\"target/jvm/classes\\" "
              + ":runtime/dependencies {:maven {org.apache.commons/commons-lang3 "
              + "{:version \\"3.12.0\\"}}}}}}}");''',
)
replace_once(
    test,
    '      assertTrue(Files.isRegularFile(root.resolve("target/classes/demo_app/Bridge.class")));',
    '      assertTrue(Files.isRegularFile(root.resolve("target/jvm/classes/demo_app/Bridge.class")));',
)

marker = '''public class MainTest {
  @Test
  public void projectCommandsRunAndReportStandardTestResults() throws Exception {'''
addition = '''public class MainTest {
  @Test
  public void newProjectScaffoldsJvmRuntimeProfile() throws Exception {
    String name = "runtime-profile-app-" + Long.toUnsignedString(System.nanoTime());
    Path root = Path.of(name);
    ByteArrayOutputStream output = new ByteArrayOutputStream();
    ByteArrayOutputStream error = new ByteArrayOutputStream();
    try {
      assertEquals(
          0,
          Main.run(
              new String[] {"new", name},
              new PrintStream(output, true, StandardCharsets.UTF_8),
              new PrintStream(error, true, StandardCharsets.UTF_8)));
      String manifest = Files.readString(root.resolve("project.edn"));
      assertTrue(manifest.contains(":project/runtime-profiles"));
      assertTrue(manifest.contains(":runtime/native-source-paths [\\"src-java\\"]"));
      assertTrue(manifest.contains(":runtime/target-path \\"target/jvm/classes\\""));
      assertTrue(!manifest.contains(":jvm/source-paths"));
      HaraProject project = HaraProject.read(root.resolve("project.edn"));
      assertEquals(root.toAbsolutePath().normalize().resolve("src-java"), project.jvmSourcePaths().get(0));
    } finally {
      if (Files.exists(root)) {
        Files.walk(root)
            .sorted(Comparator.reverseOrder())
            .forEach(
                path -> {
                  try {
                    Files.deleteIfExists(path);
                  } catch (Exception ignored) {
                  }
                });
      }
    }
  }

  @Test
  public void projectSyncUsesEffectiveHaraDependencies() throws Exception {
    Path root = Files.createTempDirectory("hara-cli-runtime-dependencies-");
    try {
      Files.writeString(
          root.resolve("project.edn"),
          "{:hara/type :project :hara/version \\"1.0.0\\" :project/id demo-app "
              + ":project/version \\"0.1.0\\" :project/source-paths [] :project/test-paths [] "
              + ":project/extension-paths [] :project/capabilities #{} :project/dependencies {} "
              + ":project/runtime-profiles {:jvm {:runtime/dependencies {:hara "
              + "{\\"hara:hara/remote\\" {:version \\"^1.0.0\\"}}}}}}}");
      ByteArrayOutputStream output = new ByteArrayOutputStream();
      ByteArrayOutputStream error = new ByteArrayOutputStream();
      int status =
          Main.run(
              new String[] {"--project", root.toString(), "sync"},
              new PrintStream(output, true, StandardCharsets.UTF_8),
              new PrintStream(error, true, StandardCharsets.UTF_8));
      assertEquals(1, status);
      assertTrue(error.toString(StandardCharsets.UTF_8).contains("1 active Hara dependencies"));
    } finally {
      Files.walk(root)
          .sorted(Comparator.reverseOrder())
          .forEach(
              path -> {
                try {
                  Files.deleteIfExists(path);
                } catch (Exception ignored) {
                }
              });
    }
  }

  @Test
  public void projectCommandsRunAndReportStandardTestResults() throws Exception {'''
replace_once(test, marker, addition)
