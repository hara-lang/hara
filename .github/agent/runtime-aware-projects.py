from pathlib import Path
import re


def replace_once(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected one match in {path}, found {count}: {old[:120]!r}")
    file.write_text(text.replace(old, new, 1))


def replace_regex(path: str, pattern: str, replacement: str) -> None:
    file = Path(path)
    text = file.read_text()
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.S)
    if count != 1:
        raise SystemExit(f"expected one regex match in {path}, found {count}: {pattern[:120]!r}")
    file.write_text(updated)


# ---------------------------------------------------------------------------
# Portable HAL project normalization
# ---------------------------------------------------------------------------

hal = "core/lib/src/tool/project.hal"
required = """(def +required-project-keys+
  [:hara/type :hara/version :project/id :project/version
   :project/source-paths :project/test-paths
   :project/extension-paths :project/capabilities])
"""
runtime_hal = r'''

(def +legacy-runtime-keys+
  {:jvm/source-paths
   [:project/runtime-profiles :jvm :runtime/native-source-paths]
   :jvm/dependencies
   [:project/runtime-profiles :jvm :runtime/dependencies :maven]
   :jvm/target-path
   [:project/runtime-profiles :jvm :runtime/target-path]})

(defn reject-legacy-runtime-keys
  [document]
  (let [legacy-key
        (if (has? document :jvm/source-paths)
          :jvm/source-paths
          (if (has? document :jvm/dependencies)
            :jvm/dependencies
            (if (has? document :jvm/target-path)
              :jvm/target-path
              nil)))]
    (if legacy-key
      (let [replacement (get +legacy-runtime-keys+ legacy-key)]
        (throw
         (ex-info
          (str "project.edn " legacy-key
               " is no longer supported; use " replacement)
          {:type :legacy-runtime-key
           :key legacy-key
           :replacement replacement})))
      document)))

(defn- runtime-path-vector
  [container key label]
  (let [value (or (get container key) [])]
    (if (vector? value)
      value
      (throw
       (ex-info
        (str label " must be a vector of project-relative path strings")
        {:type :invalid-runtime-profile
         :key key
         :value value})))))

(defn- runtime-map
  [value label]
  (let [value (or value {})]
    (if (map? value)
      value
      (throw
       (ex-info
        (str label " must be a map")
        {:type :invalid-runtime-profile
         :value value})))))

(defn runtime-profile
  [document runtime]
  (if-not (or (= runtime :jvm) (= runtime :rust))
    (throw
     (ex-info
      (str "unsupported project runtime profile " runtime)
      {:type :invalid-runtime-profile
       :runtime runtime})))
  (let [profiles (runtime-map (:project/runtime-profiles document)
                              ":project/runtime-profiles")
        profile (or (get profiles runtime) {})]
    (runtime-map profile (str ":project/runtime-profiles " runtime))))

(defn- merge-runtime-hara-dependencies
  [shared additions runtime]
  (reduce
   (fn [output [coordinate declaration]]
     (if (and (has? output coordinate)
              (not= (get output coordinate) declaration))
       (throw
        (ex-info
         (str "conflicting Hara dependency requirements for " coordinate
              " in runtime profile " runtime)
         {:type :runtime-profile-conflict
          :runtime runtime
          :coordinate coordinate
          :shared (get output coordinate)
          :runtime-requirement declaration}))
       (assoc output coordinate declaration)))
   shared
   additions))

(defn resolve-runtime-profile
  [document runtime]
  (let [document (reject-legacy-runtime-keys document)
        profile (runtime-profile document runtime)
        dependency-groups
        (runtime-map (:runtime/dependencies profile)
                     (str ":runtime/dependencies for " runtime))
        shared-hara
        (runtime-map (:project/dependencies document)
                     ":project/dependencies")
        runtime-hara
        (runtime-map (:hara dependency-groups)
                     (str ":runtime/dependencies :hara for " runtime))
        maven
        (runtime-map (:maven dependency-groups)
                     (str ":runtime/dependencies :maven for " runtime))]
    {:runtime runtime
     :source-paths
     (vec (concat
           (runtime-path-vector document
                                :project/source-paths
                                ":project/source-paths")
           (runtime-path-vector profile
                                :runtime/source-paths
                                ":runtime/source-paths")))
     :test-paths
     (vec (concat
           (runtime-path-vector document
                                :project/test-paths
                                ":project/test-paths")
           (runtime-path-vector profile
                                :runtime/test-paths
                                ":runtime/test-paths")))
     :extension-paths
     (vec (concat
           (runtime-path-vector document
                                :project/extension-paths
                                ":project/extension-paths")
           (runtime-path-vector profile
                                :runtime/extension-paths
                                ":runtime/extension-paths")))
     :native-source-paths
     (runtime-path-vector profile
                          :runtime/native-source-paths
                          ":runtime/native-source-paths")
     :target-path (:runtime/target-path profile)
     :hara-dependencies
     (merge-runtime-hara-dependencies shared-hara runtime-hara runtime)
     :maven-dependencies maven}))

(defn select-runtime
  [document runtime]
  (let [resolved (resolve-runtime-profile document runtime)]
    (assoc document
           :project/runtime runtime
           :project/source-paths (:source-paths resolved)
           :project/test-paths (:test-paths resolved)
           :project/extension-paths (:extension-paths resolved)
           :project/native-source-paths (:native-source-paths resolved)
           :project/runtime-target-path (:target-path resolved)
           :project/dependencies (:hara-dependencies resolved)
           :project/maven-dependencies (:maven-dependencies resolved))))

(defn- validate-runtime-profiles
  [document]
  (let [document (reject-legacy-runtime-keys document)
        profiles (runtime-map (:project/runtime-profiles document)
                              ":project/runtime-profiles")]
    (reduce
     (fn [output runtime]
       (do
         (resolve-runtime-profile document runtime)
         output))
     document
     (keys profiles))))
'''
replace_once(hal, required, required + runtime_hal)

old_validate = """(defn validate
  [document]
  (let [missing (missing-keys document +required-project-keys+)]
    (if (seq missing)
      (throw (ex-info "project.edn is missing required keys"
                      {:missing missing})))
    (if-not (= :project (:hara/type document))
      (throw (ex-info "project.edn :hara/type must be :project"
                      {:value (:hara/type document)})))
    (production/normalize-project document)))
"""
new_validate = """(defn validate
  [document]
  (let [document (validate-runtime-profiles document)
        missing (missing-keys document +required-project-keys+)]
    (if (seq missing)
      (throw (ex-info "project.edn is missing required keys"
                      {:missing missing})))
    (if-not (= :project (:hara/type document))
      (throw (ex-info "project.edn :hara/type must be :project"
                      {:value (:hara/type document)})))
    (production/normalize-project document)))
"""
replace_once(hal, old_validate, new_validate)

old_read = """(defn read-project
  [path]
  (let [text (native/invoke :files/local-read {:path path})
        document (read-edn text)]
    (assoc (validate document)
           :project/root (parent-path path)
           :project/file path)))
"""
new_read = """(defn read-project
  ([path] (read-project path {}))
  ([path options]
   (let [text (native/invoke :files/local-read {:path path})
         document (read-edn text)
         project (assoc (validate document)
                        :project/root (parent-path path)
                        :project/file path)]
     (if (:runtime options)
       (select-runtime project (:runtime options))
       project))))
"""
replace_once(hal, old_read, new_read)

old_resources = """(defn declared-resources
  [project]
  (let [paths (files-in project (:project/source-paths project) ".hal")]
    (reduce (fn [resources path]
              (let [source (read-text path)
                    namespace (declared-namespace source)]
                (if namespace
                  (assoc resources namespace source)
                  (throw (ex-info "HAL project source has no namespace"
                                  {:path path})))))
            {} paths)))
"""
new_resources = """(defn declared-resources
  [project]
  (let [paths (files-in project (:project/source-paths project) ".hal")]
    (reduce (fn [resources path]
              (let [source (read-text path)
                    namespace (declared-namespace source)]
                (if-not namespace
                  (throw (ex-info "HAL project source has no namespace"
                                  {:path path})))
                (if (has? resources namespace)
                  (throw (ex-info "duplicate namespace in effective project profile"
                                  {:type :duplicate-namespace
                                   :namespace namespace
                                   :path path})))
                (assoc resources namespace source)))
            {} paths)))
"""
replace_once(hal, old_resources, new_resources)

hal_tests = "core/lib/test/tool/project_test.hal"
file = Path(hal_tests)
text = file.read_text()
if not text.endswith("]\n"):
    raise SystemExit("tool.project test vector did not end as expected")
extra_hal_tests = r'''
 (test-check "runtime profiles merge shared and JVM-specific collections"
  (project/resolve-runtime-profile
   {:project/source-paths ["src"]
    :project/test-paths ["test"]
    :project/extension-paths ["extensions"]
    :project/dependencies
    {"hara:hara/base" {:version "^1.0.0"}}
    :project/runtime-profiles
    {:jvm
     {:runtime/source-paths ["src-jvm"]
      :runtime/test-paths ["test-jvm"]
      :runtime/extension-paths ["extensions-jvm"]
      :runtime/native-source-paths ["src-java"]
      :runtime/target-path "target/jvm/classes"
      :runtime/dependencies
      {:maven
       {org.postgresql/postgresql {:version "42.7.7"}}}}}}
   :jvm)
  {:runtime :jvm
   :source-paths ["src" "src-jvm"]
   :test-paths ["test" "test-jvm"]
   :extension-paths ["extensions" "extensions-jvm"]
   :native-source-paths ["src-java"]
   :target-path "target/jvm/classes"
   :hara-dependencies
   {"hara:hara/base" {:version "^1.0.0"}}
   :maven-dependencies
   {org.postgresql/postgresql {:version "42.7.7"}}})

 (test-check "runtime selection preserves build profiles and merges Hara dependencies"
  (let [selected
        (project/select-runtime
         {:project/source-paths ["src"]
          :project/test-paths []
          :project/extension-paths []
          :project/dependencies
          {"hara:hara/base" {:version "^1.0.0"}}
          :project/profiles
          {:production {:profile/language :hara}}
          :project/runtime-profiles
          {:rust
           {:runtime/source-paths ["src-rust"]
            :runtime/dependencies
            {:hara
             {"hara:hara/crypto" {:version "^1.0.0"}}}}}}
         :rust)]
    [(:project/runtime selected)
     (:project/source-paths selected)
     (:project/dependencies selected)
     (:project/profiles selected)])
  [:rust
   ["src" "src-rust"]
   {"hara:hara/base" {:version "^1.0.0"}
    "hara:hara/crypto" {:version "^1.0.0"}}
   {:production {:profile/language :hara}}])

 (test-check "legacy JVM keys report their runtime-profile replacement"
  (try
    (do
      (project/resolve-runtime-profile
       {:jvm/source-paths ["src-java"]}
       :jvm)
      nil)
    (catch Throwable error
      (:replacement (ex-data error))))
  [:project/runtime-profiles :jvm :runtime/native-source-paths])

 (test-check "shared and runtime Hara requirements fail closed on conflict"
  (try
    (do
      (project/resolve-runtime-profile
       {:project/dependencies
        {"hara:hara/crypto" {:version "^1.0.0"}}
        :project/runtime-profiles
        {:rust
         {:runtime/dependencies
          {:hara
           {"hara:hara/crypto" {:version "^2.0.0"}}}}}}
       :rust)
      nil)
    (catch Throwable error
      (:coordinate (ex-data error))))
  "hara:hara/crypto")
'''
file.write_text(text[:-2] + extra_hal_tests + "]\n")


# ---------------------------------------------------------------------------
# Native Rust project loader
# ---------------------------------------------------------------------------

rust = "core/rust/src/project.rs"
replace_once(
    rust,
    """    pub source_paths: Vec<PathBuf>,
    pub test_paths: Vec<PathBuf>,
    pub extension_paths: Vec<PathBuf>,
    pub capabilities: Vec<String>,
""",
    """    /// Effective native-Rust paths (shared paths followed by :rust additions).
    pub source_paths: Vec<PathBuf>,
    pub test_paths: Vec<PathBuf>,
    pub extension_paths: Vec<PathBuf>,
    pub shared_source_paths: Vec<PathBuf>,
    pub shared_test_paths: Vec<PathBuf>,
    pub shared_extension_paths: Vec<PathBuf>,
    pub runtime_profiles: BTreeMap<String, RuntimeProfile>,
    pub active_runtime: String,
    pub native_source_paths: Vec<PathBuf>,
    pub runtime_target_path: Option<PathBuf>,
    pub maven_dependencies: BTreeMap<String, String>,
    pub capabilities: Vec<String>,
""",
)
replace_once(
    rust,
    """    pub profiles: BTreeMap<String, ProjectProfile>,
    pub dependencies: BTreeMap<String, String>,
""",
    """    pub profiles: BTreeMap<String, ProjectProfile>,
    /// Effective native-Rust Hara dependencies.
    pub dependencies: BTreeMap<String, String>,
    pub shared_dependencies: BTreeMap<String, String>,
""",
)
replace_once(
    rust,
    """#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedProfile {
""",
    """#[derive(Debug, Clone, PartialEq, Default)]
pub struct RuntimeProfile {
    pub source_paths: Vec<PathBuf>,
    pub test_paths: Vec<PathBuf>,
    pub extension_paths: Vec<PathBuf>,
    pub native_source_paths: Vec<PathBuf>,
    pub target_path: Option<PathBuf>,
    pub hara_dependencies: BTreeMap<String, String>,
    pub maven_dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedRuntimeProfile {
    pub runtime: String,
    pub source_paths: Vec<PathBuf>,
    pub test_paths: Vec<PathBuf>,
    pub extension_paths: Vec<PathBuf>,
    pub native_source_paths: Vec<PathBuf>,
    pub target_path: Option<PathBuf>,
    pub hara_dependencies: BTreeMap<String, String>,
    pub maven_dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedProfile {
""",
)
replace_once(
    rust,
    """impl Project {
    /// Resolves a named runnable target without assigning any meaning to its
""",
    """impl Project {
    /// Resolves the shared project declaration with one host runtime overlay.
    pub fn resolve_runtime_profile(
        &self,
        runtime: &str,
    ) -> Result<ResolvedRuntimeProfile, String> {
        resolve_runtime_profile_values(
            runtime,
            &self.shared_source_paths,
            &self.shared_test_paths,
            &self.shared_extension_paths,
            &self.shared_dependencies,
            &self.runtime_profiles,
        )
    }

    /// Resolves a named runnable target without assigning any meaning to its
""",
)
replace_once(
    rust,
    """    let entries = map(&form, "project.edn must be an EDN map")?;
    for key in REQUIRED {
""",
    """    let entries = map(&form, "project.edn must be an EDN map")?;
    reject_legacy_runtime_keys(entries)?;
    for key in REQUIRED {
""",
)
replace_once(
    rust,
    """    let source_paths = paths(
        lookup(entries, "project/source-paths").unwrap(),
        "project/source-paths",
    )?;
    let test_paths = paths(
        lookup(entries, "project/test-paths").unwrap(),
        "project/test-paths",
    )?;
    let extension_paths = paths(
        lookup(entries, "project/extension-paths").unwrap(),
        "project/extension-paths",
    )?;
""",
    """    let shared_source_paths = paths(
        lookup(entries, "project/source-paths").unwrap(),
        "project/source-paths",
    )?;
    let shared_test_paths = paths(
        lookup(entries, "project/test-paths").unwrap(),
        "project/test-paths",
    )?;
    let shared_extension_paths = paths(
        lookup(entries, "project/extension-paths").unwrap(),
        "project/extension-paths",
    )?;
""",
)
replace_once(
    rust,
    """    let dependencies = lookup(entries, "project/dependencies")
        .map(dependencies)
        .transpose()?
        .unwrap_or_default();
    let extensions = lookup(entries, "project/extensions")
""",
    """    let shared_dependencies = lookup(entries, "project/dependencies")
        .map(dependencies)
        .transpose()?
        .unwrap_or_default();
    let runtime_profiles = lookup(entries, "project/runtime-profiles")
        .map(runtime_profiles)
        .transpose()?
        .unwrap_or_default();
    let active = resolve_runtime_profile_values(
        "rust",
        &shared_source_paths,
        &shared_test_paths,
        &shared_extension_paths,
        &shared_dependencies,
        &runtime_profiles,
    )?;
    let source_paths = active.source_paths.clone();
    let test_paths = active.test_paths.clone();
    let extension_paths = active.extension_paths.clone();
    let dependencies = active.hara_dependencies.clone();
    let native_source_paths = active.native_source_paths.clone();
    let runtime_target_path = active.target_path.clone();
    let maven_dependencies = active.maven_dependencies.clone();
    let extensions = lookup(entries, "project/extensions")
""",
)
replace_once(
    rust,
    """        source_paths,
        test_paths,
        extension_paths,
        capabilities,
""",
    """        source_paths,
        test_paths,
        extension_paths,
        shared_source_paths,
        shared_test_paths,
        shared_extension_paths,
        runtime_profiles,
        active_runtime: "rust".into(),
        native_source_paths,
        runtime_target_path,
        maven_dependencies,
        capabilities,
""",
)
replace_once(
    rust,
    """        profiles,
        dependencies,
        extensions,
""",
    """        profiles,
        dependencies,
        shared_dependencies,
        extensions,
""",
)

old_register = """/// Registers namespaces from `:project/source-paths` for runtime `require`.
pub fn register_sources(project: &Project, runtime: &mut Runtime) -> Result<(), String> {
    for path in files_in(&project.root, &project.source_paths)? {
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let namespace = declared_namespace(&source)
            .map_err(|error| format!("{}: {error}", path.display()))?
            .ok_or_else(|| format!("{} does not declare an ns or ns+ namespace", path.display()))?;
        runtime.register_resource(&namespace, &source);
    }
    Ok(())
}
"""
new_register = """/// Registers namespaces from the automatically selected native Rust profile.
pub fn register_sources(project: &Project, runtime: &mut Runtime) -> Result<(), String> {
    let mut resources = Vec::new();
    let mut declarations = BTreeMap::new();
    for path in files_in(&project.root, &project.source_paths)? {
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let namespace = declared_namespace(&source)
            .map_err(|error| format!("{}: {error}", path.display()))?
            .ok_or_else(|| format!("{} does not declare an ns or ns+ namespace", path.display()))?;
        if let Some(previous) = declarations.insert(namespace.clone(), path.clone()) {
            return Err(format!(
                "duplicate namespace {namespace} in effective :rust profile: {} and {}",
                previous.display(),
                path.display()
            ));
        }
        resources.push((namespace, source));
    }
    for (namespace, source) in resources {
        runtime.register_resource(&namespace, &source);
    }
    Ok(())
}
"""
replace_once(rust, old_register, new_register)

runtime_rust_helpers = r'''
fn reject_legacy_runtime_keys(entries: &[(Form, Form)]) -> Result<(), String> {
    for (key, replacement) in [
        (
            "jvm/source-paths",
            ":project/runtime-profiles :jvm :runtime/native-source-paths",
        ),
        (
            "jvm/dependencies",
            ":project/runtime-profiles :jvm :runtime/dependencies :maven",
        ),
        (
            "jvm/target-path",
            ":project/runtime-profiles :jvm :runtime/target-path",
        ),
    ] {
        if lookup(entries, key).is_some() {
            return Err(format!(
                "project.edn :{key} is no longer supported; use {replacement}"
            ));
        }
    }
    Ok(())
}

fn runtime_profiles(form: &Form) -> Result<BTreeMap<String, RuntimeProfile>, String> {
    let mut output = BTreeMap::new();
    for (key, value) in map(
        form,
        "project.edn :project/runtime-profiles must be an EDN map",
    )? {
        let runtime = identifier(key, "runtime profile name")?;
        if runtime != "jvm" && runtime != "rust" {
            return Err(format!("unsupported project runtime profile {runtime:?}"));
        }
        let entries = map(value, "runtime profile must be an EDN map")?;
        let source_paths = lookup(entries, "runtime/source-paths")
            .map(|value| paths(value, "runtime/source-paths"))
            .transpose()?
            .unwrap_or_default();
        let test_paths = lookup(entries, "runtime/test-paths")
            .map(|value| paths(value, "runtime/test-paths"))
            .transpose()?
            .unwrap_or_default();
        let extension_paths = lookup(entries, "runtime/extension-paths")
            .map(|value| paths(value, "runtime/extension-paths"))
            .transpose()?
            .unwrap_or_default();
        let native_source_paths = lookup(entries, "runtime/native-source-paths")
            .map(|value| paths(value, "runtime/native-source-paths"))
            .transpose()?
            .unwrap_or_default();
        let target_path = lookup(entries, "runtime/target-path")
            .map(|value| {
                relative_path(
                    &string(value, "runtime/target-path")?,
                    "runtime/target-path",
                )
            })
            .transpose()?;
        let (hara_dependencies, maven_dependencies) =
            match lookup(entries, "runtime/dependencies") {
                None => (BTreeMap::new(), BTreeMap::new()),
                Some(value) => {
                    let groups = map(value, "runtime :runtime/dependencies must be an EDN map")?;
                    let hara = lookup(groups, "hara")
                        .map(dependencies)
                        .transpose()?
                        .unwrap_or_default();
                    let maven = lookup(groups, "maven")
                        .map(maven_dependencies)
                        .transpose()?
                        .unwrap_or_default();
                    (hara, maven)
                }
            };
        let profile = RuntimeProfile {
            source_paths,
            test_paths,
            extension_paths,
            native_source_paths,
            target_path,
            hara_dependencies,
            maven_dependencies,
        };
        if output.insert(runtime.clone(), profile).is_some() {
            return Err(format!("duplicate project runtime profile {runtime:?}"));
        }
    }
    Ok(output)
}

fn resolve_runtime_profile_values(
    runtime: &str,
    shared_source_paths: &[PathBuf],
    shared_test_paths: &[PathBuf],
    shared_extension_paths: &[PathBuf],
    shared_dependencies: &BTreeMap<String, String>,
    runtime_profiles: &BTreeMap<String, RuntimeProfile>,
) -> Result<ResolvedRuntimeProfile, String> {
    if runtime != "jvm" && runtime != "rust" {
        return Err(format!("unsupported project runtime profile {runtime:?}"));
    }
    let profile = runtime_profiles.get(runtime).cloned().unwrap_or_default();
    let mut hara_dependencies = shared_dependencies.clone();
    for (coordinate, requirement) in &profile.hara_dependencies {
        if let Some(shared) = hara_dependencies.get(coordinate) {
            if shared != requirement {
                return Err(format!(
                    "conflicting Hara dependency requirements for {coordinate} in :{runtime}: {shared:?} and {requirement:?}"
                ));
            }
        }
        hara_dependencies.insert(coordinate.clone(), requirement.clone());
    }
    let mut source_paths = shared_source_paths.to_vec();
    source_paths.extend(profile.source_paths.iter().cloned());
    let mut test_paths = shared_test_paths.to_vec();
    test_paths.extend(profile.test_paths.iter().cloned());
    let mut extension_paths = shared_extension_paths.to_vec();
    extension_paths.extend(profile.extension_paths.iter().cloned());
    Ok(ResolvedRuntimeProfile {
        runtime: runtime.into(),
        source_paths,
        test_paths,
        extension_paths,
        native_source_paths: profile.native_source_paths,
        target_path: profile.target_path,
        hara_dependencies,
        maven_dependencies: profile.maven_dependencies,
    })
}

fn maven_dependencies(form: &Form) -> Result<BTreeMap<String, String>, String> {
    let mut output = BTreeMap::new();
    for (key, value) in map(form, "runtime Maven dependencies must be an EDN map")? {
        let coordinate = scalar(key, "Maven dependency coordinate")?;
        let mut parts = coordinate.split('/');
        if !matches!(
            (parts.next(), parts.next(), parts.next()),
            (Some(group), Some(artifact), None) if !group.is_empty() && !artifact.is_empty()
        ) {
            return Err(format!("invalid Maven dependency coordinate {coordinate:?}"));
        }
        let declaration = map(value, "Maven dependency declaration must be an EDN map")?;
        let version = lookup(declaration, "version")
            .ok_or_else(|| format!("Maven dependency {coordinate} is missing :version"))
            .and_then(|value| string(value, "Maven dependency :version"))?;
        if version.is_empty()
            || version
                .chars()
                .any(|value| matches!(value, '[' | ']' | '(' | ')' | ',' | '*'))
        {
            return Err(format!(
                "Maven dependency {coordinate} requires an exact version"
            ));
        }
        if output.insert(coordinate.clone(), version).is_some() {
            return Err(format!("duplicate Maven dependency {coordinate}"));
        }
    }
    Ok(output)
}

'''
replace_once(
    rust,
    "fn project_profiles(form: &Form) -> Result<BTreeMap<String, ProjectProfile>, String> {",
    runtime_rust_helpers
    + "fn project_profiles(form: &Form) -> Result<BTreeMap<String, ProjectProfile>, String> {",
)

rust_tests = "core/rust/src/project/tests.rs"
file = Path(rust_tests)
text = file.read_text()
extra_rust_tests = r'''

#[test]
fn selects_rust_runtime_profile_and_can_resolve_jvm_overlay() {
    let root = temp("runtime-profiles");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("project.edn"),
        "{:hara/type :project :hara/version \"1.0.0\" :project/id demo/app :project/version \"1.0.0\" :project/source-paths [\"src\"] :project/test-paths [\"test\"] :project/extension-paths [\"extensions\"] :project/capabilities #{} :project/dependencies {\"hara:hara/base\" {:version \"^1.0.0\"}} :project/profiles {:production {:profile/language :hara}} :project/runtime-profiles {:rust {:runtime/source-paths [\"src-rust\"] :runtime/test-paths [\"test-rust\"] :runtime/extension-paths [\"extensions-rust\"] :runtime/dependencies {:hara {\"hara:hara/crypto\" {:version \"^1.0.0\"}}}} :jvm {:runtime/source-paths [\"src-jvm\"] :runtime/native-source-paths [\"src-java\"] :runtime/target-path \"target/jvm/classes\" :runtime/dependencies {:maven {org.postgresql/postgresql {:version \"42.7.7\"}}}}}}",
    )
    .unwrap();
    let project = read(&root).unwrap();
    assert_eq!(project.active_runtime, "rust");
    assert_eq!(
        project.source_paths,
        vec![PathBuf::from("src"), PathBuf::from("src-rust")]
    );
    assert_eq!(
        project.test_paths,
        vec![PathBuf::from("test"), PathBuf::from("test-rust")]
    );
    assert_eq!(
        project.extension_paths,
        vec![PathBuf::from("extensions"), PathBuf::from("extensions-rust")]
    );
    assert_eq!(project.dependencies["hara:hara/base"], "^1.0.0");
    assert_eq!(project.dependencies["hara:hara/crypto"], "^1.0.0");
    assert!(project.profiles.contains_key("production"));

    let jvm = project.resolve_runtime_profile("jvm").unwrap();
    assert_eq!(
        jvm.source_paths,
        vec![PathBuf::from("src"), PathBuf::from("src-jvm")]
    );
    assert_eq!(jvm.native_source_paths, vec![PathBuf::from("src-java")]);
    assert_eq!(
        jvm.target_path,
        Some(PathBuf::from("target/jvm/classes"))
    );
    assert_eq!(
        jvm.maven_dependencies["org.postgresql/postgresql"],
        "42.7.7"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_legacy_jvm_project_keys_and_conflicting_runtime_dependencies() {
    let root = temp("runtime-invalid");
    fs::create_dir_all(&root).unwrap();
    let prefix = "{:hara/type :project :hara/version \"1.0.0\" :project/id demo/app :project/version \"1.0.0\" :project/source-paths [] :project/test-paths [] :project/extension-paths [] :project/capabilities #{} ";
    fs::write(
        root.join("project.edn"),
        format!("{prefix}:jvm/source-paths [\"src-java\"]}}"),
    )
    .unwrap();
    let legacy = read(&root).unwrap_err();
    assert!(legacy.contains(":project/runtime-profiles :jvm :runtime/native-source-paths"));

    fs::write(
        root.join("project.edn"),
        format!("{prefix}:project/dependencies {{\"hara:hara/crypto\" {{:version \"^1.0.0\"}}}} :project/runtime-profiles {{:rust {{:runtime/dependencies {{:hara {{\"hara:hara/crypto\" {{:version \"^2.0.0\"}}}}}}}}}}}}"),
    )
    .unwrap();
    let conflict = read(&root).unwrap_err();
    assert!(conflict.contains("conflicting Hara dependency requirements"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn runtime_namespace_alternatives_are_isolated_but_effective_duplicates_fail() {
    let root = temp("runtime-namespaces");
    fs::create_dir_all(root.join("src-rust/demo")).unwrap();
    fs::create_dir_all(root.join("src-jvm/demo")).unwrap();
    fs::create_dir_all(root.join("src/demo")).unwrap();
    fs::write(
        root.join("project.edn"),
        "{:hara/type :project :hara/version \"1.0.0\" :project/id demo/app :project/version \"1.0.0\" :project/source-paths [\"src\"] :project/test-paths [] :project/extension-paths [] :project/capabilities #{} :project/runtime-profiles {:rust {:runtime/source-paths [\"src-rust\"]} :jvm {:runtime/source-paths [\"src-jvm\"]}}}",
    )
    .unwrap();
    fs::write(
        root.join("src-rust/demo/adapter.hal"),
        "(ns demo.adapter) (def runtime :rust)",
    )
    .unwrap();
    fs::write(
        root.join("src-jvm/demo/adapter.hal"),
        "(ns demo.adapter) (def runtime :jvm)",
    )
    .unwrap();
    let project = read(&root).unwrap();
    let mut runtime = Runtime::new();
    register_sources(&project, &mut runtime).unwrap();

    fs::write(
        root.join("src/demo/adapter.hal"),
        "(ns demo.adapter) (def runtime :shared)",
    )
    .unwrap();
    let project = read(&root).unwrap();
    let mut runtime = Runtime::new();
    assert!(register_sources(&project, &mut runtime)
        .unwrap_err()
        .contains("duplicate namespace demo.adapter"));
    fs::remove_dir_all(root).unwrap();
}
'''
file.write_text(text + extra_rust_tests)


# ---------------------------------------------------------------------------
# JVM project loader
# ---------------------------------------------------------------------------

java = "core/java/src/main/java/hara/truffle/HaraProject.java"
replace_once(java, "import java.util.LinkedHashSet;\n", "import java.util.LinkedHashMap;\nimport java.util.LinkedHashSet;\n")
replace_once(
    java,
    """  private final java.util.List<Path> extensionPaths;
  private final java.util.List<JvmDependency> jvmDependencies;
""",
    """  private final java.util.List<Path> extensionPaths;
  private final Map<String, String> haraDependencies;
  private final java.util.List<JvmDependency> jvmDependencies;
""",
)
replace_once(
    java,
    """      java.util.List<Path> extensionPaths,
      java.util.List<JvmDependency> jvmDependencies,
""",
    """      java.util.List<Path> extensionPaths,
      Map<String, String> haraDependencies,
      java.util.List<JvmDependency> jvmDependencies,
""",
)
replace_once(
    java,
    """    this.extensionPaths = java.util.List.copyOf(extensionPaths);
    this.jvmDependencies = java.util.List.copyOf(jvmDependencies);
""",
    """    this.extensionPaths = java.util.List.copyOf(extensionPaths);
    this.haraDependencies = Map.copyOf(haraDependencies);
    this.jvmDependencies = java.util.List.copyOf(jvmDependencies);
""",
)

project_edn_block = r'''      if (PROJECT_FILE.equals(descriptor.getFileName().toString())) {
        if (!(form instanceof IMapType<?, ?> options)
            || !(lookup(options, "project/id") instanceof Symbol projectName)) {
          throw new HaraException("project.edn expects a map with :project/id");
        }
        rejectLegacyRuntimeKeys(options, PROJECT_FILE);
        Path root = descriptor.toAbsolutePath().normalize().getParent();
        java.util.List<Path> sharedSourcePaths =
            paths(
                root,
                lookup(options, "project/source-paths"),
                "project/source-paths",
                java.util.List.of("src"),
                PROJECT_FILE);
        java.util.List<Path> sharedTestPaths =
            paths(
                root,
                lookup(options, "project/test-paths"),
                "project/test-paths",
                java.util.List.of("test"),
                PROJECT_FILE);
        java.util.List<Path> sharedExtensionPaths =
            paths(
                root,
                lookup(options, "project/extension-paths"),
                "project/extension-paths",
                java.util.List.of("extensions"),
                PROJECT_FILE);
        RuntimeProfile runtime = runtimeProfile(root, options, "jvm", PROJECT_FILE);
        Map<String, String> sharedHara =
            haraDependencies(lookup(options, "project/dependencies"), PROJECT_FILE);
        Map<String, String> effectiveHara =
            mergeHaraDependencies(sharedHara, runtime.haraDependencies(), "jvm");
        return new HaraProject(
            root,
            descriptor,
            projectName,
            lookup(options, "project/version") instanceof String value ? value : null,
            lookup(options, "project/main") instanceof Symbol value ? value : null,
            mergePaths(sharedSourcePaths, runtime.sourcePaths()),
            mergePaths(sharedTestPaths, runtime.testPaths()),
            mergePaths(sharedExtensionPaths, runtime.extensionPaths()),
            effectiveHara,
            runtime.mavenDependencies(),
            runtime.nativeSourcePaths(),
            runtime.targetPath() == null
                ? root.resolve("target/jvm/classes")
                : runtime.targetPath(),
            capabilities(lookup(options, "project/capabilities"), PROJECT_FILE));
      }
      if (!(form instanceof List'''
replace_regex(
    java,
    r'''      if \(PROJECT_FILE\.equals\(descriptor\.getFileName\(\)\.toString\(\)\) \{.*?\n      \}\n      if \(!\(form instanceof List''',
    project_edn_block,
)

# The legacy project.hal constructor now supplies the added Hara-dependency field.
replace_once(
    java,
    """          java.util.List.of(root.resolve("extensions")),
          java.util.List.of(),
          java.util.List.of(),
""",
    """          java.util.List.of(root.resolve("extensions")),
          Map.of(),
          java.util.List.of(),
          java.util.List.of(),
""",
)

replace_once(
    java,
    """      if (!(form instanceof IMapType<?, ?> options)
          || !(lookup(options, "hara/type") instanceof Keyword type)
""",
    """      if (!(form instanceof IMapType<?, ?> options)
          || !(lookup(options, "hara/type") instanceof Keyword type)
""",
)
replace_once(
    java,
    """      if (!(lookup(options, "project/version") instanceof String version)
          || !version.matches(
              "^(0|[1-9][0-9]*)\\\\.(0|[1-9][0-9]*)\\\\.(0|[1-9][0-9]*)(?:[-+][0-9A-Za-z.-]+)?$"))
        throw new HaraException("project.edn :project/version is not SemVer");
      Object dependencies = lookup(options, "project/dependencies");
""",
    """      if (!(lookup(options, "project/version") instanceof String version)
          || !version.matches(
              "^(0|[1-9][0-9]*)\\\\.(0|[1-9][0-9]*)\\\\.(0|[1-9][0-9]*)(?:[-+][0-9A-Za-z.-]+)?$"))
        throw new HaraException("project.edn :project/version is not SemVer");
      rejectLegacyRuntimeKeys(options, PROJECT_FILE);
      RuntimeProfile runtime = runtimeProfile(root, options, "jvm", PROJECT_FILE);
      mergeHaraDependencies(
          haraDependencies(lookup(options, "project/dependencies"), PROJECT_FILE),
          runtime.haraDependencies(),
          "jvm");
      Object dependencies = lookup(options, "project/dependencies");
""",
)

replace_once(
    java,
    """  java.util.List<JvmDependency> jvmDependencies() {
    return jvmDependencies;
  }
""",
    """  Map<String, String> haraDependencies() {
    return haraDependencies;
  }

  java.util.List<JvmDependency> jvmDependencies() {
    return jvmDependencies;
  }
""",
)

runtime_java_helpers = r'''
  private record RuntimeProfile(
      java.util.List<Path> sourcePaths,
      java.util.List<Path> testPaths,
      java.util.List<Path> extensionPaths,
      java.util.List<Path> nativeSourcePaths,
      Path targetPath,
      Map<String, String> haraDependencies,
      java.util.List<JvmDependency> mavenDependencies) {}

  private static RuntimeProfile runtimeProfile(
      Path root, IMapType<?, ?> project, String runtime, String descriptor) {
    Object declaredProfiles = lookup(project, "project/runtime-profiles");
    if (declaredProfiles == null) {
      return new RuntimeProfile(
          java.util.List.of(),
          java.util.List.of(),
          java.util.List.of(),
          java.util.List.of(),
          null,
          Map.of(),
          java.util.List.of());
    }
    if (!(declaredProfiles instanceof IMapType<?, ?> profiles)) {
      throw new HaraException(descriptor + " :project/runtime-profiles must be a map");
    }
    Object declaredProfile = lookup(profiles, runtime);
    if (declaredProfile == null) {
      return new RuntimeProfile(
          java.util.List.of(),
          java.util.List.of(),
          java.util.List.of(),
          java.util.List.of(),
          null,
          Map.of(),
          java.util.List.of());
    }
    if (!(declaredProfile instanceof IMapType<?, ?> profile)) {
      throw new HaraException(
          descriptor + " :project/runtime-profiles :" + runtime + " must be a map");
    }
    Object declaredDependencies = lookup(profile, "runtime/dependencies");
    IMapType<?, ?> dependencyGroups;
    if (declaredDependencies == null) {
      dependencyGroups = null;
    } else if (declaredDependencies instanceof IMapType<?, ?> map) {
      dependencyGroups = map;
    } else {
      throw new HaraException(
          descriptor + " :runtime/dependencies for :" + runtime + " must be a map");
    }
    Object target = lookup(profile, "runtime/target-path");
    return new RuntimeProfile(
        paths(
            root,
            lookup(profile, "runtime/source-paths"),
            "runtime/source-paths",
            java.util.List.of(),
            descriptor),
        paths(
            root,
            lookup(profile, "runtime/test-paths"),
            "runtime/test-paths",
            java.util.List.of(),
            descriptor),
        paths(
            root,
            lookup(profile, "runtime/extension-paths"),
            "runtime/extension-paths",
            java.util.List.of(),
            descriptor),
        paths(
            root,
            lookup(profile, "runtime/native-source-paths"),
            "runtime/native-source-paths",
            java.util.List.of(),
            descriptor),
        target == null
            ? null
            : path(root, target, "runtime/target-path", null, descriptor),
        haraDependencies(
            dependencyGroups == null ? null : lookup(dependencyGroups, "hara"), descriptor),
        mavenDependencies(
            dependencyGroups == null ? null : lookup(dependencyGroups, "maven"), descriptor));
  }

  private static java.util.List<Path> mergePaths(
      java.util.List<Path> shared, java.util.List<Path> runtime) {
    ArrayList<Path> paths = new ArrayList<>(shared);
    paths.addAll(runtime);
    return java.util.List.copyOf(paths);
  }

  private static Map<String, String> mergeHaraDependencies(
      Map<String, String> shared, Map<String, String> runtime, String profile) {
    LinkedHashMap<String, String> dependencies = new LinkedHashMap<>(shared);
    for (Map.Entry<String, String> entry : runtime.entrySet()) {
      String existing = dependencies.get(entry.getKey());
      if (existing != null && !existing.equals(entry.getValue())) {
        throw new HaraException(
            "Conflicting Hara dependency requirements for "
                + entry.getKey()
                + " in :"
                + profile
                + ": "
                + existing
                + " and "
                + entry.getValue());
      }
      dependencies.put(entry.getKey(), entry.getValue());
    }
    return Map.copyOf(dependencies);
  }

  private static Map<String, String> haraDependencies(Object value, String descriptor) {
    if (value == null) return Map.of();
    if (!(value instanceof IMapType<?, ?> entries)) {
      throw new HaraException(descriptor + " Hara dependencies must be a map");
    }
    LinkedHashMap<String, String> dependencies = new LinkedHashMap<>();
    Iterator<?> iterator = entries.iterator();
    while (iterator.hasNext()) {
      Map.Entry<?, ?> entry = (Map.Entry<?, ?>) iterator.next();
      String coordinate = haraCoordinate(entry.getKey(), descriptor);
      if (!(entry.getValue() instanceof IMapType<?, ?> declaration)
          || !(lookup(declaration, "version") instanceof String version)
          || version.isBlank()) {
        throw new HaraException(
            descriptor + " Hara dependency " + coordinate + " requires :version");
      }
      if (dependencies.put(coordinate, version) != null) {
        throw new HaraException(descriptor + " duplicate Hara dependency " + coordinate);
      }
    }
    return Map.copyOf(dependencies);
  }

  private static String haraCoordinate(Object value, String descriptor) {
    String coordinate;
    if (value instanceof Symbol symbol) {
      coordinate = symbol.display();
    } else if (value instanceof String text) {
      coordinate = text;
    } else {
      throw new HaraException(descriptor + " Hara dependency coordinates must be symbols or strings");
    }
    if (coordinate.startsWith("official:")) {
      coordinate = "hara:" + coordinate.substring("official:".length());
    } else if (!coordinate.contains(":")) {
      coordinate = "hara:" + coordinate;
    }
    if (!coordinate.matches("[a-z0-9_.-]+:[a-z0-9_.-]+/[a-z0-9_.-]+")) {
      throw new HaraException(descriptor + " invalid Hara dependency coordinate " + coordinate);
    }
    return coordinate;
  }

  private static void rejectLegacyRuntimeKeys(IMapType<?, ?> options, String descriptor) {
    for (Map.Entry<String, String> legacy :
        Map.of(
                "jvm/source-paths",
                ":project/runtime-profiles :jvm :runtime/native-source-paths",
                "jvm/dependencies",
                ":project/runtime-profiles :jvm :runtime/dependencies :maven",
                "jvm/target-path",
                ":project/runtime-profiles :jvm :runtime/target-path")
            .entrySet()) {
      if (lookup(options, legacy.getKey()) != null) {
        throw new HaraException(
            descriptor
                + " :"
                + legacy.getKey()
                + " is no longer supported; use "
                + legacy.getValue());
      }
    }
  }

'''
replace_once(
    java,
    """  @SuppressWarnings("rawtypes")
  private static Object lookup(IMapType<?, ?> map, String key) {
""",
    runtime_java_helpers
    + """  @SuppressWarnings("rawtypes")
  private static Object lookup(IMapType<?, ?> map, String key) {
""",
)

# path(...) must tolerate a null default when runtime target path is optional.
replace_once(
    java,
    """    Object selected = value == null ? defaultValue : value;
    if (!(selected instanceof String entry) || entry.isBlank()) {
""",
    """    Object selected = value == null ? defaultValue : value;
    if (!(selected instanceof String entry) || entry.isBlank()) {
""",
)

new_maven_method = r'''  private static java.util.List<JvmDependency> mavenDependencies(
      Object value, String descriptor) {
    if (value == null) return java.util.List.of();
    if (!(value instanceof IMapType<?, ?> entries)) {
      throw new HaraException(descriptor + " runtime Maven dependencies must be a map");
    }
    ArrayList<JvmDependency> dependencies = new ArrayList<>();
    LinkedHashSet<String> ids = new LinkedHashSet<>();
    Iterator<?> iterator = entries.iterator();
    while (iterator.hasNext()) {
      Map.Entry<?, ?> entry = (Map.Entry<?, ?>) iterator.next();
      Object idValue = entry.getKey();
      String id;
      if (idValue instanceof Symbol symbol) {
        id = symbol.display();
      } else if (idValue instanceof String text) {
        id = text.replace(':', '/');
      } else {
        throw new HaraException(
            descriptor + " Maven dependency coordinates must be symbols or strings");
      }
      if (!id.matches("[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+")) {
        throw new HaraException(descriptor + " invalid Maven dependency coordinate " + id);
      }
      if (!(entry.getValue() instanceof IMapType<?, ?> declaration)
          || !(lookup(declaration, "version") instanceof String version)
          || !version.matches("[A-Za-z0-9][A-Za-z0-9._+-]*")) {
        throw new HaraException(
            descriptor + " Maven dependency " + id + " requires an exact version");
      }
      if (!ids.add(id)) {
        throw new HaraException(descriptor + " duplicate Maven dependency " + id);
      }
      dependencies.add(new JvmDependency(id, version));
    }
    return java.util.List.copyOf(dependencies);
  }

'''
replace_regex(
    java,
    r'''  private static java\.util\.List<JvmDependency> jvmDependencies\(Object value, String descriptor\) \{.*?\n  \}\n\n  private static Set<String> capabilities''',
    new_maven_method + "  private static Set<String> capabilities",
)

java_tests = "core/java/src/test/java/hara/truffle/HaraProjectTest.java"
new_java_tests = r'''  @Test
  public void parsesJvmRuntimeProfileAndMergesEffectivePaths() throws Exception {
    Path root = Files.createTempDirectory("hara-project-jvm");
    Files.writeString(
        root.resolve("project.edn"),
        "{:project/id sample :project/source-paths [\"src\"] :project/test-paths [\"test\"] "
            + ":project/extension-paths [\"extensions\"] :project/capabilities #{:jvm/reflection} "
            + ":project/dependencies {\"hara:hara/base\" {:version \"^1.0.0\"}} "
            + ":project/profiles {:production {:profile/language :hara}} "
            + ":project/runtime-profiles {:jvm {"
            + ":runtime/source-paths [\"src-jvm\"] :runtime/test-paths [\"test-jvm\"] "
            + ":runtime/extension-paths [\"extensions-jvm\"] "
            + ":runtime/native-source-paths [\"java-src\"] "
            + ":runtime/target-path \"build/classes\" "
            + ":runtime/dependencies {:hara {\"hara:hara/jvm\" {:version \"^1.0.0\"}} "
            + ":maven {org.apache.commons/commons-lang3 {:version \"3.12.0\"}}}}}}}");

    HaraProject project = HaraProject.read(root.resolve("project.edn"));

    assertEquals(
        java.util.List.of(root.resolve("src"), root.resolve("src-jvm")),
        project.sourcePaths());
    assertEquals(
        java.util.List.of(root.resolve("test"), root.resolve("test-jvm")),
        project.testPaths());
    assertEquals(
        java.util.List.of(root.resolve("extensions"), root.resolve("extensions-jvm")),
        project.extensionRoots());
    assertEquals(
        java.util.List.of("org.apache.commons:commons-lang3:3.12.0"),
        project.jvmDependencies().stream().map(HaraProject.JvmDependency::coordinate).toList());
    assertEquals(java.util.List.of(root.resolve("java-src")), project.jvmSourcePaths());
    assertEquals(root.resolve("build/classes"), project.jvmTargetPath());
    assertEquals("^1.0.0", project.haraDependencies().get("hara:hara/base"));
    assertEquals("^1.0.0", project.haraDependencies().get("hara:hara/jvm"));
    assertTrue(project.hasCapability("jvm/reflection"));
  }

  @Test
  public void rejectsJvmMavenDependencyRanges() throws Exception {
    Path root = Files.createTempDirectory("hara-project-jvm-invalid");
    Path descriptor = root.resolve("project.edn");
    Files.writeString(
        descriptor,
        "{:project/id sample :project/runtime-profiles {:jvm {:runtime/dependencies "
            + "{:maven {org.example/library {:version \"[1,2)\"}}}}}}}");

    HaraException error =
        assertThrows(HaraException.class, () -> HaraProject.read(descriptor));
    assertTrue(error.getMessage().contains("exact version"));
  }

  @Test
  public void rejectsLegacyJvmKeysWithReplacement() throws Exception {
    Path root = Files.createTempDirectory("hara-project-jvm-legacy");
    Path descriptor = root.resolve("project.edn");
    Files.writeString(
        descriptor,
        "{:project/id sample :jvm/source-paths [\"java-src\"]}");

    HaraException error =
        assertThrows(HaraException.class, () -> HaraProject.read(descriptor));
    assertTrue(
        error
            .getMessage()
            .contains(":project/runtime-profiles :jvm :runtime/native-source-paths"));
  }

  @Test
  public void rejectsConflictingSharedAndJvmHaraRequirements() throws Exception {
    Path root = Files.createTempDirectory("hara-project-jvm-hara-conflict");
    Path descriptor = root.resolve("project.edn");
    Files.writeString(
        descriptor,
        "{:project/id sample "
            + ":project/dependencies {\"hara:hara/crypto\" {:version \"^1.0.0\"}} "
            + ":project/runtime-profiles {:jvm {:runtime/dependencies {:hara "
            + "{\"hara:hara/crypto\" {:version \"^2.0.0\"}}}}}}}");

    HaraException error =
        assertThrows(HaraException.class, () -> HaraProject.read(descriptor));
    assertTrue(error.getMessage().contains("Conflicting Hara dependency requirements"));
  }

  @Test
  public void requiresProjectNamespacesByConvention'''
replace_regex(
    java_tests,
    r'''  @Test\n  public void parsesLeinStyleJvmDependenciesAndBuildPaths\(\) throws Exception \{.*?\n  @Test\n  public void requiresProjectNamespacesByConvention''',
    new_java_tests,
)
