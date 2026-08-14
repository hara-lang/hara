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

# The initial generator wrote raw regular-expression literals with every
# backslash doubled. Collapse only the pattern argument of replace_regex calls;
# replacement text and ordinary source literals must keep their escaping.
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
