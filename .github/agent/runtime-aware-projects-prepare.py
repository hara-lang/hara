from pathlib import Path
import re

path = Path('.github/agent/runtime-aware-projects.py')
text = path.read_text()


def replace(pattern: str, replacement: str) -> None:
    global text
    text, count = re.subn(pattern, replacement, text, count=1, flags=re.S)
    if count != 1:
        raise SystemExit(f'failed to adapt patch harness: {pattern[:80]}')


replace(
    r'required = """.*?"""\nruntime_hal =',
    '''required = """(def +required-keys+\n  [:hara/type :hara/version :project/id :project/version\n   :project/source-paths :project/test-paths :project/extension-paths\n   :project/capabilities])\n"""\nruntime_hal =''',
)

replace(
    r'old_validate = """.*?replace_once\(hal, old_validate, new_validate\)',
    '''old_validate = """(defn validate [document]\n  (let [missing (missing-keys document)]\n    (cond\n      (not (map? document))\n      (throw (ex-info "project.edn must be an EDN map" {}))\n      (not (empty? missing))\n      (throw (ex-info (str "project.edn missing required keys: " missing)\n                      {:missing missing}))\n      (not= :project (:hara/type document))\n      (throw (ex-info "project.edn :hara/type must be :project" {}))\n      :else (production/normalize-project document))))\n"""\nnew_validate = """(defn validate [document]\n  (let [document (validate-runtime-profiles document)\n        missing (missing-keys document)]\n    (cond\n      (not (map? document))\n      (throw (ex-info "project.edn must be an EDN map" {}))\n      (not (empty? missing))\n      (throw (ex-info (str "project.edn missing required keys: " missing)\n                      {:missing missing}))\n      (not= :project (:hara/type document))\n      (throw (ex-info "project.edn :hara/type must be :project" {}))\n      :else (production/normalize-project document))))\n"""\nreplace_once(hal, old_validate, new_validate)''',
)

replace(
    r'old_read = """.*?replace_once\(hal, old_read, new_read\)',
    '''old_read = """(defn read-project [manifest-path]\n  (let [document (validate (Edn/read (read-text manifest-path)))]\n    (assoc document\n           :project/root (File/parent manifest-path)\n           :project/manifest-path manifest-path)))\n"""\nnew_read = """(defn read-project\n  ([manifest-path] (read-project manifest-path {}))\n  ([manifest-path options]\n   (let [document (validate (Edn/read (read-text manifest-path)))\n         project (assoc document\n                        :project/root (File/parent manifest-path)\n                        :project/manifest-path manifest-path)]\n     (if (:runtime options)\n       (select-runtime project (:runtime options))\n       project))))\n"""\nreplace_once(hal, old_read, new_read)''',
)

replace(
    r'old_resources = """.*?replace_once\(hal, old_resources, new_resources\)',
    '''old_resources = """(defn declared-resources [project]\n  (reduce\n   (fn [resources path]\n     (let [source (read-text path)\n           namespace (framework/namespace-name source)]\n       (if (nil? namespace)\n         (throw (ex-info (str path " does not declare an ns or ns+ namespace")\n                         {:path path}))\n         (conj resources {:resource/name namespace\n                          :resource/path path\n                          :resource/source source}))))\n   []\n   (files-in project (:project/source-paths project) ".hal")))\n"""\nnew_resources = """(defn declared-resources [project]\n  (:resources\n   (reduce\n    (fn [state path]\n      (let [source (read-text path)\n            namespace (framework/namespace-name source)]\n        (if (nil? namespace)\n          (throw (ex-info (str path " does not declare an ns or ns+ namespace")\n                          {:path path})))\n        (if (has? (:names state) namespace)\n          (throw (ex-info "duplicate namespace in effective project profile"\n                          {:type :duplicate-namespace\n                           :namespace namespace\n                           :paths [(get (:names state) namespace) path]})))\n        {:names (assoc (:names state) namespace path)\n         :resources\n         (conj (:resources state)\n               {:resource/name namespace\n                :resource/path path\n                :resource/source source})}))\n    {:names {} :resources []}\n    (files-in project (:project/source-paths project) ".hal"))))\n"""\nreplace_once(hal, old_resources, new_resources)''',
)

# Make the three multiline source replacements robust to formatting drift.
helper_definition = re.compile(
    r'def replace_regex\(path: str, pattern: str, replacement: str\) -> None:\n.*?\n\n\n# ---------------------------------------------------------------------------',
    re.S,
)
new_helper = '''def replace_regex(path: str, pattern: str, replacement: str) -> None:
    file = Path(path)
    source = file.read_text()
    updated, count = re.subn(pattern, replacement, source, count=1, flags=re.S)
    if count != 1:
        if "PROJECT_FILE" in pattern:
            start_marker = "      if (PROJECT_FILE.equals(descriptor.getFileName().toString())) {"
            end_marker = "      if (!(form instanceof List"
        elif "jvmDependencies" in pattern:
            start_marker = "  private static java.util.List<JvmDependency> jvmDependencies("
            end_marker = "  private static Set<String> capabilities"
        elif path.endswith("HaraProjectTest.java"):
            start_method = source.index(
                "  public void parsesLeinStyleJvmDependenciesAndBuildPaths()"
            )
            start = source.rfind("  @Test", 0, start_method)
            end_method = source.index(
                "  public void requiresProjectNamespacesByConvention", start_method
            )
            end_marker = "  @Test" + chr(10) + "  public void requiresProjectNamespacesByConvention"
            end = source.rfind("  @Test", 0, end_method)
            if start < 0 or end < 0:
                raise SystemExit(f"cannot locate JVM project test boundaries in {path}")
            updated = source[:start] + replacement + source[end + len(end_marker):]
            file.write_text(updated)
            return
        else:
            raise SystemExit(
                f"expected one regex match in {path}, found {count}: {pattern[:120]!r}"
            )
        start = source.index(start_marker)
        end = source.index(end_marker, start)
        updated = source[:start] + replacement + source[end + len(end_marker):]
    file.write_text(updated)
'''
text, helper_count = helper_definition.subn(
    lambda _: new_helper + chr(10) + chr(10) + '# ---------------------------------------------------------------------------',
    text,
    count=1,
)
if helper_count != 1:
    raise SystemExit(f'expected one replace_regex helper, found {helper_count}')

# The initial generator also wrote raw regular-expression literals with every
# backslash doubled. Collapse only pattern arguments; structural fallbacks above
# remain authoritative if a source formatter changes whitespace again.
pattern_argument = re.compile(
    r"(replace_regex\(\s*[^,]+,\s*r''')(.*?)(''',)", re.S
)


def normalize_pattern(match: re.Match[str]) -> str:
    slash = chr(92)
    return match.group(1) + match.group(2).replace(slash * 2, slash) + match.group(3)


text, pattern_count = pattern_argument.subn(normalize_pattern, text)
if pattern_count != 3:
    raise SystemExit(f'expected three replace_regex patterns, found {pattern_count}')

path.write_text(text)
