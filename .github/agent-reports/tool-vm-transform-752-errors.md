# tool.vm transformation gate diagnostics

- Run: https://github.com/hara-lang/hara/actions/runs/32126764945
- Staging SHA: `2501f7bcc6bf01b6456263f197e9f1f37383842e`
- rust-default: `101`
- rust-vm-profile: `101`
- rust-vm-check: `0`
- hara-vm: `0`
- truffle: `1`
- hara: `2`
- layout: `1`

## rust-default

```text
450:error: couldn't read `src/kernel/../../../../hara-specs-registry/01-lang/009-halc/draft/conformance/golden/complete.halc`: No such file or directory (os error 2)
459:error: couldn't read `src/kernel/../../../../hara-specs-registry/01-lang/009-halc/draft/conformance/golden/legacy-v1.hir`: No such file or directory (os error 2)
468:error: couldn't read `src/vm/../../../../../hara-specs-registry/01-lang/010-bytecode/draft/conformance/bytecode-vm.edn`: No such file or directory (os error 2)
477:error: couldn't read `src/vm/../../../../../hara-specs-registry/00-unsorted/platform-language/draft/conformance/modules.edn`: No such file or directory (os error 2)
486:error: couldn't read `src/vm/../../../../../hara-specs-registry/00-unsorted/platform-language/draft/conformance/l0.edn`: No such file or directory (os error 2)
528:124 |     if actual_hash.as_slice() != expected_hash.as_slice() {
813:warning: function `unsupported_handoff_evidence` is never used
816:97 | fn unsupported_handoff_evidence(
892:error: could not compile `hara-wasm` (lib test) due to 5 previous errors; 35 warnings emitted
--- tail ---
warning: function `map_value_for_test` is never used
   --> src/cli_app/manifest.rs:143:15
    |
143 | pub(super) fn map_value_for_test<'a>(entries: &'a [(Form, Form)], key: &str) -> Option<&'a Form> {
    |               ^^^^^^^^^^^^^^^^^^

warning: function `parse` is never used
   --> src/core/async_value.rs:395:4
    |
395 | fn parse(source: &str) -> Result<Form, String> {
    |    ^^^^^

warning: function `iterator_map_spread` is never used
    --> src/core/operation.rs:1062:4
     |
1062 | fn iterator_map_spread(function: Rc<Function>, value: Value) -> Result<Value, String> {
     |    ^^^^^^^^^^^^^^^^^^^

warning: function `unsupported_handoff_evidence` is never used
  --> src/kernel/halc_bytecode_trace.rs:97:4
   |
97 | fn unsupported_handoff_evidence(
   |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: associated function `load` is never used
  --> src/native_extension.rs:21:12
   |
20 | impl ExtensionPackage {
   | --------------------- associated function in this implementation
21 |     pub fn load(root: &Path) -> Result<Self, String> {
   |            ^^^^

warning: function `packages_in_project` is never used
   --> src/native_extension.rs:148:4
    |
148 | fn packages_in_project(root: &Path) -> Result<Vec<ExtensionPackage>, String> {
    |    ^^^^^^^^^^^^^^^^^^^

warning: method `to_i64_exact` is never used
   --> src/numeric.rs:325:19
    |
 64 | impl ExactDecimal {
    | ----------------- method in this implementation
...
325 |     pub(crate) fn to_i64_exact(&self) -> Option<i64> {
    |                   ^^^^^^^^^^^^

warning: function `canonical_decimal` is never used
   --> src/numeric.rs:341:15
    |
341 | pub(crate) fn canonical_decimal(value: &str) -> Result<String, String> {
    |               ^^^^^^^^^^^^^^^^^

warning: function `numeric_increment` is never used
   --> src/numeric.rs:652:15
    |
652 | pub(crate) fn numeric_increment(value: &Value, delta: i64) -> Result<Value, String> {
    |               ^^^^^^^^^^^^^^^^^

warning: function `to_i32_exact` is never used
   --> src/numeric.rs:715:15
    |
715 | pub(crate) fn to_i32_exact(value: &Value) -> Result<i32, String> {
    |               ^^^^^^^^^^^^

warning: function `to_u32_exact` is never used
   --> src/numeric.rs:727:15
    |
727 | pub(crate) fn to_u32_exact(value: &Value) -> Result<u32, String> {
    |               ^^^^^^^^^^^^

warning: function `invoke_zero_arity` is never used
  --> src/project/production/bundle/load.rs:16:15
   |
16 | pub(super) fn invoke_zero_arity(runtime: &Runtime, symbol: &str) -> Result<Value, String> {
   |               ^^^^^^^^^^^^^^^^^

warning: field `plan` is never read
 --> src/project/production/bundle/model.rs:6:37
  |
5 | pub(in crate::task::production) struct ProductionBuild {
  |                                        --------------- field in this struct
6 |     pub(in crate::task::production) plan: BuildPlan,
  |                                     ^^^^
  |
  = note: `ProductionBuild` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: method `locals` is never used
   --> src/vm/frame.rs:124:19
    |
 11 | impl Frame {
    | ---------- method in this implementation
...
124 |     pub(crate) fn locals(&self) -> &[VmSlot] {
    |                   ^^^^^^

warning: `hara-wasm` (lib test) generated 35 warnings (17 duplicates)
error: could not compile `hara-wasm` (lib test) due to 5 previous errors; 35 warnings emitted
warning: build failed, waiting for other jobs to finish...
warning: `hara-wasm` (lib) generated 34 warnings (run `cargo fix --lib -p hara-wasm` to apply 11 suggestions)
```

## rust-vm-profile

```text
2:error: couldn't read `src/kernel/../../../../hara-specs-registry/01-lang/009-halc/draft/conformance/golden/complete.halc`: No such file or directory (os error 2)
11:error: couldn't read `src/kernel/../../../../hara-specs-registry/01-lang/009-halc/draft/conformance/golden/legacy-v1.hir`: No such file or directory (os error 2)
20:error: couldn't read `src/vm/../../../../../hara-specs-registry/01-lang/010-bytecode/draft/conformance/bytecode-vm.edn`: No such file or directory (os error 2)
29:error: couldn't read `src/vm/../../../../../hara-specs-registry/00-unsorted/platform-language/draft/conformance/modules.edn`: No such file or directory (os error 2)
38:error: couldn't read `src/vm/../../../../../hara-specs-registry/00-unsorted/platform-language/draft/conformance/l0.edn`: No such file or directory (os error 2)
80:124 |     if actual_hash.as_slice() != expected_hash.as_slice() {
468:error: could not compile `hara-wasm` (lib test) due to 5 previous errors; 35 warnings emitted
--- tail ---
warning: function `vm_tool_resource_option` is never used
   --> src/core/vm_tool.rs:309:4
    |
309 | fn vm_tool_resource_option(options: &Value) -> Result<Option<String>, String> {
    |    ^^^^^^^^^^^^^^^^^^^^^^^

warning: function `vm_tool_default_resource` is never used
   --> src/core/vm_tool.rs:339:4
    |
339 | fn vm_tool_default_resource(namespace: &str) -> String {
    |    ^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `parse` is never used
   --> src/core/async_value.rs:395:4
    |
395 | fn parse(source: &str) -> Result<Form, String> {
    |    ^^^^^

warning: function `iterator_map_spread` is never used
    --> src/core/operation.rs:1062:4
     |
1062 | fn iterator_map_spread(function: Rc<Function>, value: Value) -> Result<Value, String> {
     |    ^^^^^^^^^^^^^^^^^^^

warning: associated function `load` is never used
  --> src/native_extension.rs:21:12
   |
20 | impl ExtensionPackage {
   | --------------------- associated function in this implementation
21 |     pub fn load(root: &Path) -> Result<Self, String> {
   |            ^^^^

warning: function `packages_in_project` is never used
   --> src/native_extension.rs:148:4
    |
148 | fn packages_in_project(root: &Path) -> Result<Vec<ExtensionPackage>, String> {
    |    ^^^^^^^^^^^^^^^^^^^

warning: method `to_i64_exact` is never used
   --> src/numeric.rs:325:19
    |
 64 | impl ExactDecimal {
    | ----------------- method in this implementation
...
325 |     pub(crate) fn to_i64_exact(&self) -> Option<i64> {
    |                   ^^^^^^^^^^^^

warning: function `canonical_decimal` is never used
   --> src/numeric.rs:341:15
    |
341 | pub(crate) fn canonical_decimal(value: &str) -> Result<String, String> {
    |               ^^^^^^^^^^^^^^^^^

warning: function `numeric_increment` is never used
   --> src/numeric.rs:652:15
    |
652 | pub(crate) fn numeric_increment(value: &Value, delta: i64) -> Result<Value, String> {
    |               ^^^^^^^^^^^^^^^^^

warning: function `to_i32_exact` is never used
   --> src/numeric.rs:715:15
    |
715 | pub(crate) fn to_i32_exact(value: &Value) -> Result<i32, String> {
    |               ^^^^^^^^^^^^

warning: function `to_u32_exact` is never used
   --> src/numeric.rs:727:15
    |
727 | pub(crate) fn to_u32_exact(value: &Value) -> Result<u32, String> {
    |               ^^^^^^^^^^^^

warning: function `invoke_zero_arity` is never used
  --> src/project/production/bundle/load.rs:16:15
   |
16 | pub(super) fn invoke_zero_arity(runtime: &Runtime, symbol: &str) -> Result<Value, String> {
   |               ^^^^^^^^^^^^^^^^^

warning: field `plan` is never read
 --> src/project/production/bundle/model.rs:6:37
  |
5 | pub(in crate::task::production) struct ProductionBuild {
  |                                        --------------- field in this struct
6 |     pub(in crate::task::production) plan: BuildPlan,
  |                                     ^^^^
  |
  = note: `ProductionBuild` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: method `locals` is never used
   --> src/vm/frame.rs:124:19
    |
 11 | impl Frame {
    | ---------- method in this implementation
...
124 |     pub(crate) fn locals(&self) -> &[VmSlot] {
    |                   ^^^^^^

warning: `hara-wasm` (lib test) generated 35 warnings (14 duplicates)
error: could not compile `hara-wasm` (lib test) due to 5 previous errors; 35 warnings emitted
warning: build failed, waiting for other jobs to finish...
warning: `hara-wasm` (lib) generated 35 warnings (run `cargo fix --lib -p hara-wasm` to apply 11 suggestions)
```

## rust-vm-check

```text
241:124 |     if actual_hash.as_slice() != expected_hash.as_slice() {
795:260 | fn render_runtime_error(error: &str) -> Result<(), String> {
--- tail ---
   --> src/bin/hara/repl.rs:891:4
    |
891 | fn usage() {
    |    ^^^^^

warning: function `exit_error` is never used
   --> src/bin/hara/repl.rs:897:4
    |
897 | fn exit_error(message: &str, status: i32) -> ! {
    |    ^^^^^^^^^^

warning: function `history_file` is never used
 --> src/bin/hara/terminal.rs:4:15
  |
4 | pub(crate) fn history_file() -> PathBuf {
  |               ^^^^^^^^^^^^

warning: constant `DEFAULT_SPLASH` is never used
  --> src/bin/hara/terminal.rs:11:18
   |
11 | pub(crate) const DEFAULT_SPLASH: &str = r#"
   |                  ^^^^^^^^^^^^^^

warning: function `print_header` is never used
  --> src/bin/hara/terminal.rs:30:15
   |
30 | pub(crate) fn print_header(resp: &str, include_splash: bool, color: bool) {
   |               ^^^^^^^^^^^^

warning: function `rendered_splash` is never used
  --> src/bin/hara/terminal.rs:44:15
   |
44 | pub(crate) fn rendered_splash(color: bool) -> String {
   |               ^^^^^^^^^^^^^^^

warning: function `gradient` is never used
  --> src/bin/hara/terminal.rs:85:15
   |
85 | pub(crate) fn gradient(position: f64, stops: &[(i32, i32, i32)]) -> (i32, i32, i32) {
   |               ^^^^^^^^

warning: function `tagline` is never used
  --> src/bin/hara/terminal.rs:97:4
   |
97 | fn tagline(text: &str, color: bool) -> String {
   |    ^^^^^^^

warning: function `session_prompt` is never used
   --> src/bin/hara/terminal.rs:121:15
    |
121 | pub(crate) fn session_prompt(namespace: &str, color: bool) -> String {
    |               ^^^^^^^^^^^^^^

warning: function `clear_terminal` is never used
   --> src/bin/hara/terminal.rs:128:15
    |
128 | pub(crate) fn clear_terminal() {
    |               ^^^^^^^^^^^^^^

warning: function `is_terminal` is never used
   --> src/bin/hara/terminal.rs:131:15
    |
131 | pub(crate) fn is_terminal() -> bool {
    |               ^^^^^^^^^^^

warning: function `libc_isatty` is never used
   --> src/bin/hara/terminal.rs:135:11
    |
135 | unsafe fn libc_isatty(fd: i32) -> i32 {
    |           ^^^^^^^^^^^

warning: function `isatty` is never used
   --> src/bin/hara/terminal.rs:137:12
    |
137 |         fn isatty(fd: i32) -> i32;
    |            ^^^^^^

warning: `hara-wasm` (bin "hara-lite") generated 127 warnings
warning: function `run_lite` is never used
   --> src/bin/hara/cli.rs:164:15
    |
164 | pub(crate) fn run_lite(mut options: Options) -> Result<(), String> {
    |               ^^^^^^^^
    |
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `usage_lite` is never used
   --> src/bin/hara/cli.rs:218:15
    |
218 | pub(crate) fn usage_lite() {
    |               ^^^^^^^^^^

warning: function `run_file` is never used
   --> src/bin/hara/cli/project.rs:731:15
    |
731 | pub(crate) fn run_file(options: &Options, path: &str) -> Result<(), String> {
    |               ^^^^^^^^

warning: `hara-wasm` (bin "hara") generated 119 warnings (116 duplicates)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 35.83s
```

## hara-vm

```text
34:124 |     if actual_hash.as_slice() != expected_hash.as_slice() {
265:test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
271:test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
--- tail ---
warning: function `parse` is never used
   --> src/core/async_value.rs:395:4
    |
395 | fn parse(source: &str) -> Result<Form, String> {
    |    ^^^^^

warning: function `iterator_map_spread` is never used
    --> src/core/operation.rs:1062:4
     |
1062 | fn iterator_map_spread(function: Rc<Function>, value: Value) -> Result<Value, String> {
     |    ^^^^^^^^^^^^^^^^^^^

warning: associated function `load` is never used
  --> src/native_extension.rs:21:12
   |
20 | impl ExtensionPackage {
   | --------------------- associated function in this implementation
21 |     pub fn load(root: &Path) -> Result<Self, String> {
   |            ^^^^

warning: function `packages_in_project` is never used
   --> src/native_extension.rs:148:4
    |
148 | fn packages_in_project(root: &Path) -> Result<Vec<ExtensionPackage>, String> {
    |    ^^^^^^^^^^^^^^^^^^^

warning: method `to_i64_exact` is never used
   --> src/numeric.rs:325:19
    |
 64 | impl ExactDecimal {
    | ----------------- method in this implementation
...
325 |     pub(crate) fn to_i64_exact(&self) -> Option<i64> {
    |                   ^^^^^^^^^^^^

warning: function `canonical_decimal` is never used
   --> src/numeric.rs:341:15
    |
341 | pub(crate) fn canonical_decimal(value: &str) -> Result<String, String> {
    |               ^^^^^^^^^^^^^^^^^

warning: function `numeric_increment` is never used
   --> src/numeric.rs:652:15
    |
652 | pub(crate) fn numeric_increment(value: &Value, delta: i64) -> Result<Value, String> {
    |               ^^^^^^^^^^^^^^^^^

warning: function `to_i32_exact` is never used
   --> src/numeric.rs:715:15
    |
715 | pub(crate) fn to_i32_exact(value: &Value) -> Result<i32, String> {
    |               ^^^^^^^^^^^^

warning: function `to_u32_exact` is never used
   --> src/numeric.rs:727:15
    |
727 | pub(crate) fn to_u32_exact(value: &Value) -> Result<u32, String> {
    |               ^^^^^^^^^^^^

warning: function `invoke_zero_arity` is never used
  --> src/project/production/bundle/load.rs:16:15
   |
16 | pub(super) fn invoke_zero_arity(runtime: &Runtime, symbol: &str) -> Result<Value, String> {
   |               ^^^^^^^^^^^^^^^^^

warning: field `plan` is never read
 --> src/project/production/bundle/model.rs:6:37
  |
5 | pub(in crate::task::production) struct ProductionBuild {
  |                                        --------------- field in this struct
6 |     pub(in crate::task::production) plan: BuildPlan,
  |                                     ^^^^
  |
  = note: `ProductionBuild` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: method `locals` is never used
   --> src/vm/frame.rs:124:19
    |
 11 | impl Frame {
    | ---------- method in this implementation
...
124 |     pub(crate) fn locals(&self) -> &[VmSlot] {
    |                   ^^^^^^

warning: `hara-wasm` (lib) generated 35 warnings (run `cargo fix --lib -p hara-wasm` to apply 11 suggestions)
   Compiling hara-vm v0.1.6 (/home/runner/work/hara/hara/core/rust/vm-runtime)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.02s
     Running unittests src/lib.rs (core/rust/target/debug/deps/hara_vm-0cb01c74ba20a255)

running 1 test
test tests::rejects_unverified_input_without_a_compiler_dependency ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests hara_vm

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

## truffle

```text
688:[ERROR] Tests run: 5, Failures: 1, Errors: 0, Skipped: 0, Time elapsed: 2.13 s <<< FAILURE! - in hara.truffle.ToolVmLibraryTest
690:org.junit.ComparisonFailure: expected:<[[:validate :inspect :transform :disassemble]]> but was:<[hara.lang.data.Vector$Standard<:validate,:inspect,:transform,:disassemble>]>
697:[ERROR]   ToolVmLibraryTest.providerReportsOnlyTheImplementedTransformation:47 expected:<[[:validate :inspect :transform :disassemble]]> but was:<[hara.lang.data.Vector$Standard<:validate,:inspect,:transform,:disassemble>]>
699:[ERROR] Tests run: 5, Failures: 1, Errors: 0, Skipped: 0
702:[INFO] BUILD FAILURE
--- tail ---
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/maven-surefire-common/3.0.0-M5/maven-surefire-common-3.0.0-M5.jar
Progress (1): 7.7/311 kBProgress (1): 16/311 kB Progress (1): 24/311 kBProgress (1): 40/311 kBProgress (1): 57/311 kBProgress (1): 73/311 kBProgress (1): 90/311 kBProgress (1): 106/311 kBProgress (1): 122/311 kBProgress (1): 139/311 kBProgress (1): 155/311 kBProgress (1): 172/311 kBProgress (1): 188/311 kBProgress (1): 204/311 kBProgress (1): 221/311 kBProgress (1): 237/311 kBProgress (1): 253/311 kBProgress (1): 270/311 kBProgress (1): 286/311 kBProgress (1): 303/311 kBProgress (1): 311 kB                        Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/maven-surefire-common/3.0.0-M5/maven-surefire-common-3.0.0-M5.jar (311 kB at 16 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-api/3.0.0-M5/surefire-api-3.0.0-M5.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-logger-api/3.0.0-M5/surefire-logger-api-3.0.0-M5.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-extensions-api/3.0.0-M5/surefire-extensions-api-3.0.0-M5.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-booter/3.0.0-M5/surefire-booter-3.0.0-M5.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-extensions-spi/3.0.0-M5/surefire-extensions-spi-3.0.0-M5.jar
Progress (1): 7.7/144 kBProgress (1): 16/144 kB Progress (1): 24/144 kBProgress (1): 40/144 kBProgress (1): 57/144 kBProgress (1): 73/144 kBProgress (1): 90/144 kBProgress (1): 106/144 kBProgress (1): 122/144 kBProgress (2): 122/144 kB | 3.2/106 kBProgress (3): 122/144 kB | 3.2/106 kB | 7.7/14 kBProgress (3): 122/144 kB | 3.2/106 kB | 14 kB    Progress (3): 122/144 kB | 11/106 kB | 14 kB Progress (3): 139/144 kB | 11/106 kB | 14 kBProgress (3): 144 kB | 11/106 kB | 14 kB    Progress (3): 144 kB | 28/106 kB | 14 kBProgress (3): 144 kB | 44/106 kB | 14 kBProgress (3): 144 kB | 61/106 kB | 14 kBProgress (3): 144 kB | 77/106 kB | 14 kB                                        Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-api/3.0.0-M5/surefire-api-3.0.0-M5.jar (144 kB at 7.2 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven-toolchain/3.0-alpha-2/maven-toolchain-3.0-alpha-2.jar
Progress (2): 93/106 kB | 14 kB                               Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-logger-api/3.0.0-M5/surefire-logger-api-3.0.0-M5.jar (14 kB at 682 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-artifact-transfer/0.11.0/maven-artifact-transfer-0.11.0.jar
Progress (1): 106 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-booter/3.0.0-M5/surefire-booter-3.0.0-M5.jar (106 kB at 5.3 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-common-artifact-filters/3.1.0/maven-common-artifact-filters-3.1.0.jar
Progress (1): 7.3/7.9 kBProgress (1): 7.9 kB                        Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-extensions-spi/3.0.0-M5/surefire-extensions-spi-3.0.0-M5.jar (7.9 kB at 361 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven-plugin-api/3.0/maven-plugin-api-3.0.jar
Progress (1): 7.7/24 kBProgress (1): 16/24 kB Progress (1): 24 kB                      Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-extensions-api/3.0.0-M5/surefire-extensions-api-3.0.0-M5.jar (24 kB at 857 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/commons-codec/commons-codec/1.11/commons-codec-1.11.jar
Progress (1): 3.2/49 kBProgress (2): 3.2/49 kB | 7.7/36 kBProgress (2): 3.2/49 kB | 16/36 kB Progress (2): 3.2/49 kB | 24/36 kBProgress (2): 3.2/49 kB | 36 kB   Progress (2): 20/49 kB | 36 kB Progress (2): 36/49 kB | 36 kBProgress (2): 49 kB | 36 kB                              Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven-toolchain/3.0-alpha-2/maven-toolchain-3.0-alpha-2.jar (36 kB at 950 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-java/1.0.5/plexus-java-1.0.5.jar
Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven-plugin-api/3.0/maven-plugin-api-3.0.jar (49 kB at 1.3 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/com/thoughtworks/qdox/qdox/2.0-M9/qdox-2.0-M9.jar
Progress (1): 7.7/128 kBProgress (1): 11/128 kB Progress (1): 28/128 kBProgress (1): 44/128 kBProgress (1): 61/128 kBProgress (1): 77/128 kBProgress (1): 93/128 kBProgress (1): 97/128 kBProgress (2): 97/128 kB | 7.7/61 kBProgress (2): 97/128 kB | 11/61 kB Progress (2): 113/128 kB | 11/61 kBProgress (2): 128 kB | 11/61 kB    Progress (2): 128 kB | 28/61 kBProgress (2): 128 kB | 44/61 kB                               Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-artifact-transfer/0.11.0/maven-artifact-transfer-0.11.0.jar (128 kB at 3.3 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-shared-utils/3.0.0-M4/surefire-shared-utils-3.0.0-M4.jar
Progress (1): 61/61 kBProgress (1): 61 kB                      Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-common-artifact-filters/3.1.0/maven-common-artifact-filters-3.1.0.jar (61 kB at 1.5 MB/s)
Progress (1): 7.7/335 kBProgress (1): 15/335 kB Progress (1): 32/335 kBProgress (1): 48/335 kBProgress (1): 65/335 kBProgress (1): 81/335 kBProgress (1): 97/335 kBProgress (1): 114/335 kBProgress (1): 130/335 kBProgress (1): 147/335 kBProgress (1): 163/335 kBProgress (1): 179/335 kBProgress (1): 196/335 kBProgress (1): 212/335 kBProgress (1): 228/335 kBProgress (1): 245/335 kBProgress (1): 261/335 kBProgress (1): 278/335 kBProgress (1): 294/335 kBProgress (1): 310/335 kBProgress (1): 327/335 kBProgress (1): 335 kB                        Downloaded from central: https://repo.maven.apache.org/maven2/commons-codec/commons-codec/1.11/commons-codec-1.11.jar (335 kB at 6.2 MB/s)
Progress (1): 7.7/52 kBProgress (1): 16/52 kB Progress (2): 16/52 kB | 0/1.9 MBProgress (2): 16/52 kB | 0/1.9 MBProgress (2): 32/52 kB | 0/1.9 MBProgress (2): 49/52 kB | 0/1.9 MBProgress (2): 49/52 kB | 0/1.9 MBProgress (2): 52 kB | 0/1.9 MB   Progress (2): 52 kB | 0/1.9 MBProgress (2): 52 kB | 0.1/1.9 MB                                Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-java/1.0.5/plexus-java-1.0.5.jar (52 kB at 931 kB/s)
Progress (1): 0.1/1.9 MBProgress (1): 0.1/1.9 MBProgress (1): 0.1/1.9 MBProgress (1): 0.1/1.9 MBProgress (2): 0.1/1.9 MB | 7.7/317 kBProgress (2): 0.1/1.9 MB | 7.7/317 kBProgress (2): 0.1/1.9 MB | 11/317 kB Progress (2): 0.1/1.9 MB | 11/317 kBProgress (2): 0.1/1.9 MB | 28/317 kBProgress (2): 0.2/1.9 MB | 28/317 kBProgress (2): 0.2/1.9 MB | 44/317 kBProgress (2): 0.2/1.9 MB | 44/317 kBProgress (2): 0.2/1.9 MB | 61/317 kBProgress (2): 0.2/1.9 MB | 61/317 kBProgress (2): 0.2/1.9 MB | 77/317 kBProgress (2): 0.2/1.9 MB | 77/317 kBProgress (2): 0.2/1.9 MB | 93/317 kBProgress (2): 0.2/1.9 MB | 110/317 kBProgress (2): 0.2/1.9 MB | 110/317 kBProgress (2): 0.2/1.9 MB | 115/317 kBProgress (2): 0.2/1.9 MB | 115/317 kBProgress (2): 0.2/1.9 MB | 131/317 kBProgress (2): 0.3/1.9 MB | 131/317 kBProgress (2): 0.3/1.9 MB | 147/317 kBProgress (2): 0.3/1.9 MB | 164/317 kBProgress (2): 0.3/1.9 MB | 164/317 kBProgress (2): 0.3/1.9 MB | 180/317 kBProgress (2): 0.3/1.9 MB | 180/317 kBProgress (2): 0.3/1.9 MB | 197/317 kBProgress (2): 0.3/1.9 MB | 197/317 kBProgress (2): 0.3/1.9 MB | 213/317 kBProgress (2): 0.3/1.9 MB | 213/317 kBProgress (2): 0.3/1.9 MB | 229/317 kBProgress (2): 0.3/1.9 MB | 229/317 kBProgress (2): 0.3/1.9 MB | 246/317 kBProgress (2): 0.4/1.9 MB | 246/317 kBProgress (2): 0.4/1.9 MB | 262/317 kBProgress (2): 0.4/1.9 MB | 262/317 kBProgress (2): 0.4/1.9 MB | 279/317 kBProgress (2): 0.4/1.9 MB | 295/317 kBProgress (2): 0.4/1.9 MB | 295/317 kBProgress (2): 0.4/1.9 MB | 295/317 kBProgress (2): 0.4/1.9 MB | 295/317 kBProgress (2): 0.4/1.9 MB | 295/317 kBProgress (2): 0.5/1.9 MB | 295/317 kBProgress (2): 0.5/1.9 MB | 295/317 kBProgress (2): 0.5/1.9 MB | 295/317 kBProgress (2): 0.5/1.9 MB | 311/317 kBProgress (2): 0.5/1.9 MB | 317 kB    Progress (2): 0.5/1.9 MB | 317 kBProgress (2): 0.5/1.9 MB | 317 kB                                 Downloaded from central: https://repo.maven.apache.org/maven2/com/thoughtworks/qdox/qdox/2.0-M9/qdox-2.0-M9.jar (317 kB at 5.2 MB/s)
Progress (1): 0.5/1.9 MBProgress (1): 0.6/1.9 MBProgress (1): 0.6/1.9 MBProgress (1): 0.6/1.9 MBProgress (1): 0.6/1.9 MBProgress (1): 0.6/1.9 MBProgress (1): 0.6/1.9 MBProgress (1): 0.7/1.9 MBProgress (1): 0.7/1.9 MBProgress (1): 0.7/1.9 MBProgress (1): 0.7/1.9 MBProgress (1): 0.7/1.9 MBProgress (1): 0.7/1.9 MBProgress (1): 0.8/1.9 MBProgress (1): 0.8/1.9 MBProgress (1): 0.8/1.9 MBProgress (1): 0.8/1.9 MBProgress (1): 0.8/1.9 MBProgress (1): 0.8/1.9 MBProgress (1): 0.9/1.9 MBProgress (1): 0.9/1.9 MBProgress (1): 0.9/1.9 MBProgress (1): 0.9/1.9 MBProgress (1): 0.9/1.9 MBProgress (1): 0.9/1.9 MBProgress (1): 1.0/1.9 MBProgress (1): 1.0/1.9 MBProgress (1): 1.0/1.9 MBProgress (1): 1.0/1.9 MBProgress (1): 1.0/1.9 MBProgress (1): 1.0/1.9 MBProgress (1): 1.0/1.9 MBProgress (1): 1.1/1.9 MBProgress (1): 1.1/1.9 MBProgress (1): 1.1/1.9 MBProgress (1): 1.1/1.9 MBProgress (1): 1.1/1.9 MBProgress (1): 1.1/1.9 MBProgress (1): 1.2/1.9 MBProgress (1): 1.2/1.9 MBProgress (1): 1.2/1.9 MBProgress (1): 1.2/1.9 MBProgress (1): 1.2/1.9 MBProgress (1): 1.2/1.9 MBProgress (1): 1.3/1.9 MBProgress (1): 1.3/1.9 MBProgress (1): 1.3/1.9 MBProgress (1): 1.3/1.9 MBProgress (1): 1.3/1.9 MBProgress (1): 1.3/1.9 MBProgress (1): 1.4/1.9 MBProgress (1): 1.4/1.9 MBProgress (1): 1.4/1.9 MBProgress (1): 1.4/1.9 MBProgress (1): 1.4/1.9 MBProgress (1): 1.4/1.9 MBProgress (1): 1.5/1.9 MBProgress (1): 1.5/1.9 MBProgress (1): 1.5/1.9 MBProgress (1): 1.5/1.9 MBProgress (1): 1.5/1.9 MBProgress (1): 1.5/1.9 MBProgress (1): 1.6/1.9 MBProgress (1): 1.6/1.9 MBProgress (1): 1.6/1.9 MBProgress (1): 1.6/1.9 MBProgress (1): 1.6/1.9 MBProgress (1): 1.6/1.9 MBProgress (1): 1.7/1.9 MBProgress (1): 1.7/1.9 MBProgress (1): 1.7/1.9 MBProgress (1): 1.7/1.9 MBProgress (1): 1.7/1.9 MBProgress (1): 1.7/1.9 MBProgress (1): 1.8/1.9 MBProgress (1): 1.8/1.9 MBProgress (1): 1.8/1.9 MBProgress (1): 1.8/1.9 MBProgress (1): 1.8/1.9 MBProgress (1): 1.8/1.9 MBProgress (1): 1.9/1.9 MBProgress (1): 1.9 MB                        Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-shared-utils/3.0.0-M4/surefire-shared-utils-3.0.0-M4.jar (1.9 MB at 25 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-junit4/3.0.0-M5/surefire-junit4-3.0.0-M5.jar
Progress (1): 7.7/17 kBProgress (1): 11/17 kB Progress (1): 17 kB                      Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-junit4/3.0.0-M5/surefire-junit4-3.0.0-M5.jar (17 kB at 750 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-junit4/3.0.0-M5/surefire-junit4-3.0.0-M5.pom
Progress (1): 873 BProgress (1): 2.7 kBProgress (1): 2.9 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-junit4/3.0.0-M5/surefire-junit4-3.0.0-M5.pom (2.9 kB at 151 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-providers/3.0.0-M5/surefire-providers-3.0.0-M5.pom
Progress (1): 844 BProgress (1): 2.5 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-providers/3.0.0-M5/surefire-providers-3.0.0-M5.pom (2.5 kB at 133 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/junit/junit/4.0/junit-4.0.pom
Progress (1): 210 B                   Downloaded from central: https://repo.maven.apache.org/maven2/junit/junit/4.0/junit-4.0.pom (210 B at 12 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-junit4/3.0.0-M5/common-junit4-3.0.0-M5.pom
Progress (1): 859 BProgress (1): 2.8 kBProgress (1): 2.9 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-junit4/3.0.0-M5/common-junit4-3.0.0-M5.pom (2.9 kB at 128 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-junit3/3.0.0-M5/common-junit3-3.0.0-M5.pom
Progress (1): 873 BProgress (1): 2.6 kBProgress (1): 2.6 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-junit3/3.0.0-M5/common-junit3-3.0.0-M5.pom (2.6 kB at 130 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-java5/3.0.0-M5/common-java5-3.0.0-M5.pom
Progress (1): 872 BProgress (1): 2.6 kBProgress (1): 2.6 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-java5/3.0.0-M5/common-java5-3.0.0-M5.pom (2.6 kB at 109 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/hamcrest/hamcrest-library/1.3/hamcrest-library-1.3.pom
Progress (1): 820 B                   Downloaded from central: https://repo.maven.apache.org/maven2/org/hamcrest/hamcrest-library/1.3/hamcrest-library-1.3.pom (820 B at 34 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/easytesting/fest-assert/1.4/fest-assert-1.4.pom
Progress (1): 1.3 kBProgress (1): 2.4 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/easytesting/fest-assert/1.4/fest-assert-1.4.pom (2.4 kB at 103 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/easytesting/fest/1.0.8/fest-1.0.8.pom
Progress (1): 990 BProgress (1): 4.1 kBProgress (1): 7.3 kBProgress (1): 11 kB Progress (1): 12 kB                   Downloaded from central: https://repo.maven.apache.org/maven2/org/easytesting/fest/1.0.8/fest-1.0.8.pom (12 kB at 605 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/easytesting/fest-util/1.1.6/fest-util-1.1.6.pom
Progress (1): 1.3 kBProgress (1): 1.6 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/easytesting/fest-util/1.1.6/fest-util-1.1.6.pom (1.6 kB at 87 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-junit4/3.0.0-M5/common-junit4-3.0.0-M5.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-junit3/3.0.0-M5/common-junit3-3.0.0-M5.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-java5/3.0.0-M5/common-java5-3.0.0-M5.jar
Progress (1): 3.2/25 kBProgress (1): 20/25 kB Progress (1): 25 kB                      Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-junit4/3.0.0-M5/common-junit4-3.0.0-M5.jar (25 kB at 1.4 MB/s)
Progress (1): 7.7/16 kBProgress (1): 16 kB                       Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-java5/3.0.0-M5/common-java5-3.0.0-M5.jar (16 kB at 874 kB/s)
Progress (1): 7.7/12 kBProgress (1): 11/12 kB Progress (1): 12 kB                      Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-junit3/3.0.0-M5/common-junit3-3.0.0-M5.jar (12 kB at 419 kB/s)
[INFO] 
[INFO] -------------------------------------------------------
[INFO]  T E S T S
[INFO] -------------------------------------------------------
[INFO] Running hara.truffle.ToolVmLibraryTest
[To redirect Truffle log output to a file use one of the following options:
* '--log.file=<path>' if the option is passed using a guest language launcher.
* '-Dpolyglot.log.file=<path>' if the option is passed using the host Java launcher.
* Configure logging using the polyglot embedding API.]
[engine] WARNING: The polyglot engine uses a fallback runtime that does not support runtime compilation to native code.
Execution without runtime compilation will negatively impact the guest application performance.
The following cause was found: JVMCI is not enabled for this JVM. Enable JVMCI using -XX:+EnableJVMCI.
For more information see: https://www.graalvm.org/latest/reference-manual/embed-languages/#runtime-optimization-support.
To disable this warning use the '--engine.WarnInterpreterOnly=false' option or the '-Dpolyglot.engine.WarnInterpreterOnly=false' system property.
[ERROR] Tests run: 5, Failures: 1, Errors: 0, Skipped: 0, Time elapsed: 2.13 s <<< FAILURE! - in hara.truffle.ToolVmLibraryTest
[ERROR] hara.truffle.ToolVmLibraryTest.providerReportsOnlyTheImplementedTransformation  Time elapsed: 0.011 s  <<< FAILURE!
org.junit.ComparisonFailure: expected:<[[:validate :inspect :transform :disassemble]]> but was:<[hara.lang.data.Vector$Standard<:validate,:inspect,:transform,:disassemble>]>
	at hara.truffle.ToolVmLibraryTest.providerReportsOnlyTheImplementedTransformation(ToolVmLibraryTest.java:47)

[INFO] 
[INFO] Results:
[INFO] 
[ERROR] Failures: 
[ERROR]   ToolVmLibraryTest.providerReportsOnlyTheImplementedTransformation:47 expected:<[[:validate :inspect :transform :disassemble]]> but was:<[hara.lang.data.Vector$Standard<:validate,:inspect,:transform,:disassemble>]>
[INFO] 
[ERROR] Tests run: 5, Failures: 1, Errors: 0, Skipped: 0
[INFO] 
[INFO] ------------------------------------------------------------------------
[INFO] BUILD FAILURE
[INFO] ------------------------------------------------------------------------
[INFO] Total time:  22.300 s
[INFO] Finished at: 2026-08-18T10:29:37Z
[INFO] ------------------------------------------------------------------------
[ERROR] Failed to execute goal org.apache.maven.plugins:maven-surefire-plugin:3.0.0-M5:test (default-test) on project hara.lang: There are test failures.
[ERROR] 
[ERROR] Please refer to /home/runner/work/hara/hara/core/java/target/surefire-reports for the individual test results.
[ERROR] Please refer to dump files (if any exist) [date].dump, [date]-jvmRun[N].dump and [date].dumpstream.
[ERROR] -> [Help 1]
[ERROR] 
[ERROR] To see the full stack trace of the errors, re-run Maven with the -e switch.
[ERROR] Re-run Maven using the -X switch to enable full debug logging.
[ERROR] 
[ERROR] For more information about the errors and possible solutions, please read the following articles:
[ERROR] [Help 1] http://cwiki.apache.org/confluence/display/MAVEN/MojoFailureException
```

## hara

```text
35:124 |     if actual_hash.as_slice() != expected_hash.as_slice() {
174:warning: function `unsupported_handoff_evidence` is never used
177:97 | fn unsupported_handoff_evidence(
--- tail ---
warning: function `map_value_for_test` is never used
   --> src/cli_app/manifest.rs:143:15
    |
143 | pub(super) fn map_value_for_test<'a>(entries: &'a [(Form, Form)], key: &str) -> Option<&'a Form> {
    |               ^^^^^^^^^^^^^^^^^^

warning: function `parse` is never used
   --> src/core/async_value.rs:395:4
    |
395 | fn parse(source: &str) -> Result<Form, String> {
    |    ^^^^^

warning: function `iterator_map_spread` is never used
    --> src/core/operation.rs:1062:4
     |
1062 | fn iterator_map_spread(function: Rc<Function>, value: Value) -> Result<Value, String> {
     |    ^^^^^^^^^^^^^^^^^^^

warning: function `unsupported_handoff_evidence` is never used
  --> src/kernel/halc_bytecode_trace.rs:97:4
   |
97 | fn unsupported_handoff_evidence(
   |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: associated function `load` is never used
  --> src/native_extension.rs:21:12
   |
20 | impl ExtensionPackage {
   | --------------------- associated function in this implementation
21 |     pub fn load(root: &Path) -> Result<Self, String> {
   |            ^^^^

warning: function `packages_in_project` is never used
   --> src/native_extension.rs:148:4
    |
148 | fn packages_in_project(root: &Path) -> Result<Vec<ExtensionPackage>, String> {
    |    ^^^^^^^^^^^^^^^^^^^

warning: method `to_i64_exact` is never used
   --> src/numeric.rs:325:19
    |
 64 | impl ExactDecimal {
    | ----------------- method in this implementation
...
325 |     pub(crate) fn to_i64_exact(&self) -> Option<i64> {
    |                   ^^^^^^^^^^^^

warning: function `canonical_decimal` is never used
   --> src/numeric.rs:341:15
    |
341 | pub(crate) fn canonical_decimal(value: &str) -> Result<String, String> {
    |               ^^^^^^^^^^^^^^^^^

warning: function `numeric_increment` is never used
   --> src/numeric.rs:652:15
    |
652 | pub(crate) fn numeric_increment(value: &Value, delta: i64) -> Result<Value, String> {
    |               ^^^^^^^^^^^^^^^^^

warning: function `to_i32_exact` is never used
   --> src/numeric.rs:715:15
    |
715 | pub(crate) fn to_i32_exact(value: &Value) -> Result<i32, String> {
    |               ^^^^^^^^^^^^

warning: function `to_u32_exact` is never used
   --> src/numeric.rs:727:15
    |
727 | pub(crate) fn to_u32_exact(value: &Value) -> Result<u32, String> {
    |               ^^^^^^^^^^^^

warning: function `invoke_zero_arity` is never used
  --> src/project/production/bundle/load.rs:16:15
   |
16 | pub(super) fn invoke_zero_arity(runtime: &Runtime, symbol: &str) -> Result<Value, String> {
   |               ^^^^^^^^^^^^^^^^^

warning: field `plan` is never read
 --> src/project/production/bundle/model.rs:6:37
  |
5 | pub(in crate::task::production) struct ProductionBuild {
  |                                        --------------- field in this struct
6 |     pub(in crate::task::production) plan: BuildPlan,
  |                                     ^^^^
  |
  = note: `ProductionBuild` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: method `locals` is never used
   --> src/vm/frame.rs:124:19
    |
 11 | impl Frame {
    | ---------- method in this implementation
...
124 |     pub(crate) fn locals(&self) -> &[VmSlot] {
    |                   ^^^^^^

warning: `hara-wasm` (lib) generated 34 warnings (run `cargo fix --lib -p hara-wasm` to apply 11 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.15s
     Running `core/rust/target/debug/hara-test --root . core/lib/test/tool/vm_test.hal core/lib/test/tool/vm_provider_test.hal`
hara-test: ./core/lib/test/tool/vm_provider_test.hal: cannot read /home/runner/work/hara/hara/project.edn: No such file or directory (os error 2)
```

## layout

```text
--- tail ---
src/work.rs grew to 977 lines; recorded legacy maximum is 920
```
