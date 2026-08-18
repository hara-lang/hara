# tool.vm transformation staging report

- Run: https://github.com/hara-lang/hara/actions/runs/32125345956
- Staging SHA: `93370135c0bc9fe7dc6d8552150cdb43f1ddb5cf`
- Materialize: `success`
- Rust: `failure`
- Truffle: `failure`
- Hara: `failure`
- Bounds: `failure`
- Publish: `skipped`

## materialize

```text
From https://github.com/hara-lang/hara
 * branch              main       -> FETCH_HEAD
Switched to a new branch 'agent/tool-vm-transform-752'
branch 'agent/tool-vm-transform-752' set up to track 'origin/main'.
Applied #752 transformation implementation
```

## rust

```text
    [1m[94m|[0m         [1m[94m----[0m[1m[33m^[0m
    [1m[94m|[0m         [1m[94m|[0m
    [1m[94m|[0m         [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
   [1m[94m--> [0msrc/fiber/coroutine_tests.rs:137:9
    [1m[94m|[0m
[1m[94m137[0m [1m[94m|[0m     let mut f = EvalFiber::start(
    [1m[94m|[0m         [1m[94m----[0m[1m[33m^[0m
    [1m[94m|[0m         [1m[94m|[0m
    [1m[94m|[0m         [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
   [1m[94m--> [0msrc/fiber/coroutine_tests.rs:149:9
    [1m[94m|[0m
[1m[94m149[0m [1m[94m|[0m     let mut f = EvalFiber::start("(std.foundation.coroutine/yield 1 2 3)", HashMap::new()).unwrap();
    [1m[94m|[0m         [1m[94m----[0m[1m[33m^[0m
    [1m[94m|[0m         [1m[94m|[0m
    [1m[94m|[0m         [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
   [1m[94m--> [0msrc/fiber/coroutine_tests.rs:158:9
    [1m[94m|[0m
[1m[94m158[0m [1m[94m|[0m     let mut f = EvalFiber::start(
    [1m[94m|[0m         [1m[94m----[0m[1m[33m^[0m
    [1m[94m|[0m         [1m[94m|[0m
    [1m[94m|[0m         [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
   [1m[94m--> [0msrc/fiber/coroutine_tests.rs:177:9
    [1m[94m|[0m
[1m[94m177[0m [1m[94m|[0m     let mut f = EvalFiber::start("(std.foundation.coroutine/yield 1)", HashMap::new()).unwrap();
    [1m[94m|[0m         [1m[94m----[0m[1m[33m^[0m
    [1m[94m|[0m         [1m[94m|[0m
    [1m[94m|[0m         [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
   [1m[94m--> [0msrc/fiber/coroutine_tests.rs:183:9
    [1m[94m|[0m
[1m[94m183[0m [1m[94m|[0m     let mut f = EvalFiber::start(
    [1m[94m|[0m         [1m[94m----[0m[1m[33m^[0m
    [1m[94m|[0m         [1m[94m|[0m
    [1m[94m|[0m         [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
   [1m[94m--> [0msrc/fiber/coroutine_tests.rs:195:9
    [1m[94m|[0m
[1m[94m195[0m [1m[94m|[0m     let mut f = EvalFiber::start(
    [1m[94m|[0m         [1m[94m----[0m[1m[33m^[0m
    [1m[94m|[0m         [1m[94m|[0m
    [1m[94m|[0m         [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
   [1m[94m--> [0msrc/fiber/coroutine_tests.rs:226:9
    [1m[94m|[0m
[1m[94m226[0m [1m[94m|[0m     let mut f = EvalFiber::start(
    [1m[94m|[0m         [1m[94m----[0m[1m[33m^[0m
    [1m[94m|[0m         [1m[94m|[0m
    [1m[94m|[0m         [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
   [1m[94m--> [0msrc/fiber/coroutine_tests.rs:277:9
    [1m[94m|[0m
[1m[94m277[0m [1m[94m|[0m     let mut f = EvalFiber::start(
    [1m[94m|[0m         [1m[94m----[0m[1m[33m^[0m
    [1m[94m|[0m         [1m[94m|[0m
    [1m[94m|[0m         [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
   [1m[94m--> [0msrc/fiber/coroutine_tests.rs:292:9
    [1m[94m|[0m
[1m[94m292[0m [1m[94m|[0m     let mut f = EvalFiber::start(
    [1m[94m|[0m         [1m[94m----[0m[1m[33m^[0m
    [1m[94m|[0m         [1m[94m|[0m
    [1m[94m|[0m         [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
    [1m[94m--> [0msrc/fiber.rs:1675:13
     [1m[94m|[0m
[1m[94m1675[0m [1m[94m|[0m         let mut f = EvalFiber::start(
     [1m[94m|[0m             [1m[94m----[0m[1m[33m^[0m
     [1m[94m|[0m             [1m[94m|[0m
     [1m[94m|[0m             [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
   [1m[94m--> [0msrc/core/value.rs:881:24
    [1m[94m|[0m
[1m[94m881[0m [1m[94m|[0m             .and_then(|mut opt| {
    [1m[94m|[0m                        [1m[94m----[0m[1m[33m^^^[0m
    [1m[94m|[0m                        [1m[94m|[0m
    [1m[94m|[0m                        [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
  [1m[94m--> [0msrc/vm/fiber.rs:29:13
   [1m[94m|[0m
[1m[94m29[0m [1m[94m|[0m         let mut machine = Machine::entry(program);
   [1m[94m|[0m             [1m[94m----[0m[1m[33m^^^^^^^[0m
   [1m[94m|[0m             [1m[94m|[0m
   [1m[94m|[0m             [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
  [1m[94m--> [0msrc/vm/fiber.rs:51:13
   [1m[94m|[0m
[1m[94m51[0m [1m[94m|[0m         let mut machine = Machine::call(program, prototype, arguments, captures);
   [1m[94m|[0m             [1m[94m----[0m[1m[33m^^^^^^^[0m
   [1m[94m|[0m             [1m[94m|[0m
   [1m[94m|[0m             [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
  [1m[94m--> [0msrc/vm/machine/async_runtime.rs:36:9
   [1m[94m|[0m
[1m[94m36[0m [1m[94m|[0m         mut machine: Machine,
   [1m[94m|[0m         [1m[94m----[0m[1m[33m^^^^^^^[0m
   [1m[94m|[0m         [1m[94m|[0m
   [1m[94m|[0m         [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
   [1m[94m--> [0msrc/vm/machine.rs:551:25
    [1m[94m|[0m
[1m[94m551[0m [1m[94m|[0m                     let mut next_ip = ip;
    [1m[94m|[0m                         [1m[94m----[0m[1m[33m^^^^^^^[0m
    [1m[94m|[0m                         [1m[94m|[0m
    [1m[94m|[0m                         [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: associated function `new` is never used[0m
  [1m[94m--> [0msrc/runtime/session.rs:67:8
   [1m[94m|[0m
[1m[94m66[0m [1m[94m|[0m impl Session {
   [1m[94m|[0m [1m[94m------------[0m [1m[94massociated function in this implementation[0m
[1m[94m67[0m [1m[94m|[0m     fn new(name: &str, runtime: Runtime) -> Self {
   [1m[94m|[0m        [1m[33m^^^[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

[1m[33mwarning[0m[1m: function `map_entries_for_test` is never used[0m
   [1m[94m--> [0msrc/cli_app/manifest.rs:139:15
    [1m[94m|[0m
[1m[94m139[0m [1m[94m|[0m pub(super) fn map_entries_for_test(form: &Form) -> Result<&[(Form, Form)], String> {
    [1m[94m|[0m               [1m[33m^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `map_value_for_test` is never used[0m
   [1m[94m--> [0msrc/cli_app/manifest.rs:143:15
    [1m[94m|[0m
[1m[94m143[0m [1m[94m|[0m pub(super) fn map_value_for_test<'a>(entries: &'a [(Form, Form)], key: &str) -> Option<&'a Form> {
    [1m[94m|[0m               [1m[33m^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `parse` is never used[0m
   [1m[94m--> [0msrc/core/async_value.rs:395:4
    [1m[94m|[0m
[1m[94m395[0m [1m[94m|[0m fn parse(source: &str) -> Result<Form, String> {
    [1m[94m|[0m    [1m[33m^^^^^[0m

[1m[33mwarning[0m[1m: function `iterator_map_spread` is never used[0m
    [1m[94m--> [0msrc/core/operation.rs:1062:4
     [1m[94m|[0m
[1m[94m1062[0m [1m[94m|[0m fn iterator_map_spread(function: Rc<Function>, value: Value) -> Result<Value, String> {
     [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `unsupported_handoff_evidence` is never used[0m
  [1m[94m--> [0msrc/kernel/halc_bytecode_trace.rs:97:4
   [1m[94m|[0m
[1m[94m97[0m [1m[94m|[0m fn unsupported_handoff_evidence(
   [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: associated function `load` is never used[0m
  [1m[94m--> [0msrc/native_extension.rs:21:12
   [1m[94m|[0m
[1m[94m20[0m [1m[94m|[0m impl ExtensionPackage {
   [1m[94m|[0m [1m[94m---------------------[0m [1m[94massociated function in this implementation[0m
[1m[94m21[0m [1m[94m|[0m     pub fn load(root: &Path) -> Result<Self, String> {
   [1m[94m|[0m            [1m[33m^^^^[0m

[1m[33mwarning[0m[1m: function `packages_in_project` is never used[0m
   [1m[94m--> [0msrc/native_extension.rs:148:4
    [1m[94m|[0m
[1m[94m148[0m [1m[94m|[0m fn packages_in_project(root: &Path) -> Result<Vec<ExtensionPackage>, String> {
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: method `to_i64_exact` is never used[0m
   [1m[94m--> [0msrc/numeric.rs:325:19
    [1m[94m|[0m
[1m[94m 64[0m [1m[94m|[0m impl ExactDecimal {
    [1m[94m|[0m [1m[94m-----------------[0m [1m[94mmethod in this implementation[0m
[1m[94m...[0m
[1m[94m325[0m [1m[94m|[0m     pub(crate) fn to_i64_exact(&self) -> Option<i64> {
    [1m[94m|[0m                   [1m[33m^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `canonical_decimal` is never used[0m
   [1m[94m--> [0msrc/numeric.rs:341:15
    [1m[94m|[0m
[1m[94m341[0m [1m[94m|[0m pub(crate) fn canonical_decimal(value: &str) -> Result<String, String> {
    [1m[94m|[0m               [1m[33m^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `numeric_increment` is never used[0m
   [1m[94m--> [0msrc/numeric.rs:652:15
    [1m[94m|[0m
[1m[94m652[0m [1m[94m|[0m pub(crate) fn numeric_increment(value: &Value, delta: i64) -> Result<Value, String> {
    [1m[94m|[0m               [1m[33m^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `to_i32_exact` is never used[0m
   [1m[94m--> [0msrc/numeric.rs:715:15
    [1m[94m|[0m
[1m[94m715[0m [1m[94m|[0m pub(crate) fn to_i32_exact(value: &Value) -> Result<i32, String> {
    [1m[94m|[0m               [1m[33m^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `to_u32_exact` is never used[0m
   [1m[94m--> [0msrc/numeric.rs:727:15
    [1m[94m|[0m
[1m[94m727[0m [1m[94m|[0m pub(crate) fn to_u32_exact(value: &Value) -> Result<u32, String> {
    [1m[94m|[0m               [1m[33m^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `invoke_zero_arity` is never used[0m
  [1m[94m--> [0msrc/project/production/bundle/load.rs:16:15
   [1m[94m|[0m
[1m[94m16[0m [1m[94m|[0m pub(super) fn invoke_zero_arity(runtime: &Runtime, symbol: &str) -> Result<Value, String> {
   [1m[94m|[0m               [1m[33m^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: field `plan` is never read[0m
 [1m[94m--> [0msrc/project/production/bundle/model.rs:6:37
  [1m[94m|[0m
[1m[94m5[0m [1m[94m|[0m pub(in crate::task::production) struct ProductionBuild {
  [1m[94m|[0m                                        [1m[94m---------------[0m [1m[94mfield in this struct[0m
[1m[94m6[0m [1m[94m|[0m     pub(in crate::task::production) plan: BuildPlan,
  [1m[94m|[0m                                     [1m[33m^^^^[0m
  [1m[94m|[0m
  [1m[94m= [0m[1mnote[0m: `ProductionBuild` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

[1m[33mwarning[0m[1m: method `locals` is never used[0m
   [1m[94m--> [0msrc/vm/frame.rs:124:19
    [1m[94m|[0m
[1m[94m 11[0m [1m[94m|[0m impl Frame {
    [1m[94m|[0m [1m[94m----------[0m [1m[94mmethod in this implementation[0m
[1m[94m...[0m
[1m[94m124[0m [1m[94m|[0m     pub(crate) fn locals(&self) -> &[VmSlot] {
    [1m[94m|[0m                   [1m[33m^^^^^^[0m

[1m[33mwarning[0m: `hara-wasm` (lib test) generated 35 warnings (17 duplicates)
[1m[91merror[0m: could not compile `hara-wasm` (lib test) due to 5 previous errors; 35 warnings emitted
[1m[33mwarning[0m: build failed, waiting for other jobs to finish...
[1m[33mwarning[0m: `hara-wasm` (lib) generated 34 warnings (run `cargo fix --lib -p hara-wasm` to apply 11 suggestions)
```

## truffle

```text
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-compiler-javac/2.15.0/plexus-compiler-javac-2.15.0.pom
Progress (1): 1.3 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-compiler-javac/2.15.0/plexus-compiler-javac-2.15.0.pom (1.3 kB at 92 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-compilers/2.15.0/plexus-compilers-2.15.0.pom
Progress (1): 1.6 kBProgress (1): 1.6 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-compilers/2.15.0/plexus-compilers-2.15.0.pom (1.6 kB at 112 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-shared-utils/3.4.2/maven-shared-utils-3.4.2.jar
Progress (1): 7.7/151 kBProgress (1): 16/151 kB Progress (1): 28/151 kBProgress (1): 44/151 kBProgress (1): 61/151 kBProgress (1): 77/151 kBProgress (1): 93/151 kBProgress (1): 110/151 kBProgress (1): 126/151 kBProgress (1): 142/151 kBProgress (1): 151 kB                        Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-shared-utils/3.4.2/maven-shared-utils-3.4.2.jar (151 kB at 12 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-shared-incremental/1.1/maven-shared-incremental-1.1.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-java/1.2.0/plexus-java-1.2.0.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/ow2/asm/asm/9.6/asm-9.6.jar
Downloading from central: https://repo.maven.apache.org/maven2/com/thoughtworks/qdox/qdox/2.0.3/qdox-2.0.3.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-compiler-api/2.15.0/plexus-compiler-api-2.15.0.jar
Progress (1): 7.7/14 kBProgress (1): 14 kB                       Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-shared-incremental/1.1/maven-shared-incremental-1.1.jar (14 kB at 1.0 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-compiler-manager/2.15.0/plexus-compiler-manager-2.15.0.jar
Progress (1): 7.7/58 kBProgress (1): 15/58 kB Progress (1): 32/58 kBProgress (2): 32/58 kB | 7.7/334 kBProgress (2): 48/58 kB | 7.7/334 kBProgress (2): 48/58 kB | 16/334 kB Progress (2): 58 kB | 16/334 kB   Progress (2): 58 kB | 24/334 kBProgress (2): 58 kB | 40/334 kBProgress (2): 58 kB | 57/334 kB                               Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-java/1.2.0/plexus-java-1.2.0.jar (58 kB at 3.4 MB/s)
Progress (1): 73/334 kB                       Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-xml/3.0.0/plexus-xml-3.0.0.jar
Progress (1): 90/334 kBProgress (1): 106/334 kBProgress (1): 122/334 kBProgress (1): 139/334 kBProgress (2): 139/334 kB | 7.7/124 kBProgress (2): 139/334 kB | 7.7/124 kBProgress (2): 155/334 kB | 7.7/124 kBProgress (2): 155/334 kB | 24/124 kB Progress (2): 172/334 kB | 24/124 kBProgress (2): 172/334 kB | 40/124 kBProgress (2): 188/334 kB | 40/124 kBProgress (2): 188/334 kB | 57/124 kBProgress (2): 196/334 kB | 57/124 kBProgress (2): 196/334 kB | 73/124 kBProgress (2): 212/334 kB | 73/124 kBProgress (2): 212/334 kB | 90/124 kBProgress (2): 228/334 kB | 90/124 kBProgress (2): 228/334 kB | 106/124 kBProgress (2): 245/334 kB | 106/124 kBProgress (2): 245/334 kB | 122/124 kBProgress (2): 245/334 kB | 124 kB    Progress (2): 261/334 kB | 124 kBProgress (2): 277/334 kB | 124 kBProgress (2): 279/334 kB | 124 kBProgress (2): 295/334 kB | 124 kB                                 Downloaded from central: https://repo.maven.apache.org/maven2/org/ow2/asm/asm/9.6/asm-9.6.jar (124 kB at 6.2 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-compiler-javac/2.15.0/plexus-compiler-javac-2.15.0.jar
Progress (1): 311/334 kBProgress (1): 328/334 kBProgress (1): 334 kB                        Downloaded from central: https://repo.maven.apache.org/maven2/com/thoughtworks/qdox/qdox/2.0.3/qdox-2.0.3.jar (334 kB at 18 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-utils/4.0.0/plexus-utils-4.0.0.jar
Progress (1): 7.7/29 kBProgress (1): 11/29 kB Progress (1): 28/29 kBProgress (1): 29 kB                      Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-compiler-api/2.15.0/plexus-compiler-api-2.15.0.jar (29 kB at 1.5 MB/s)
Progress (1): 5.2 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-compiler-manager/2.15.0/plexus-compiler-manager-2.15.0.jar (5.2 kB at 249 kB/s)
Progress (1): 7.7/93 kBProgress (1): 11/93 kB Progress (1): 28/93 kBProgress (1): 44/93 kBProgress (1): 61/93 kBProgress (1): 77/93 kBProgress (1): 93 kB                      Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-xml/3.0.0/plexus-xml-3.0.0.jar (93 kB at 3.2 MB/s)
Progress (1): 7.7/26 kBProgress (1): 11/26 kB Progress (1): 26 kB                      Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-compiler-javac/2.15.0/plexus-compiler-javac-2.15.0.jar (26 kB at 829 kB/s)
Progress (1): 7.7/192 kBProgress (1): 16/192 kB Progress (1): 32/192 kBProgress (1): 49/192 kBProgress (1): 65/192 kBProgress (1): 81/192 kBProgress (1): 98/192 kBProgress (1): 114/192 kBProgress (1): 131/192 kBProgress (1): 147/192 kBProgress (1): 163/192 kBProgress (1): 180/192 kBProgress (1): 192 kB                        Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-utils/4.0.0/plexus-utils-4.0.0.jar (192 kB at 5.8 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/graalvm/truffle/truffle-dsl-processor/25.0.3/truffle-dsl-processor-25.0.3.pom
Progress (1): 1.1 kBProgress (1): 1.3 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/graalvm/truffle/truffle-dsl-processor/25.0.3/truffle-dsl-processor-25.0.3.pom (1.3 kB at 65 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/graalvm/truffle/truffle-dsl-processor/25.0.3/truffle-dsl-processor-25.0.3.jar
Progress (1): 0/3.9 MBProgress (1): 0/3.9 MBProgress (1): 0/3.9 MBProgress (1): 0/3.9 MBProgress (1): 0.1/3.9 MBProgress (1): 0.1/3.9 MBProgress (1): 0.1/3.9 MBProgress (1): 0.1/3.9 MBProgress (1): 0.1/3.9 MBProgress (1): 0.1/3.9 MBProgress (1): 0.2/3.9 MBProgress (1): 0.2/3.9 MBProgress (1): 0.2/3.9 MBProgress (1): 0.2/3.9 MBProgress (1): 0.2/3.9 MBProgress (1): 0.2/3.9 MBProgress (1): 0.3/3.9 MBProgress (1): 0.3/3.9 MBProgress (1): 0.3/3.9 MBProgress (1): 0.3/3.9 MBProgress (1): 0.3/3.9 MBProgress (1): 0.3/3.9 MBProgress (1): 0.4/3.9 MBProgress (1): 0.4/3.9 MBProgress (1): 0.4/3.9 MBProgress (1): 0.4/3.9 MBProgress (1): 0.4/3.9 MBProgress (1): 0.4/3.9 MBProgress (1): 0.4/3.9 MBProgress (1): 0.5/3.9 MBProgress (1): 0.5/3.9 MBProgress (1): 0.5/3.9 MBProgress (1): 0.5/3.9 MBProgress (1): 0.5/3.9 MBProgress (1): 0.5/3.9 MBProgress (1): 0.6/3.9 MBProgress (1): 0.6/3.9 MBProgress (1): 0.6/3.9 MBProgress (1): 0.6/3.9 MBProgress (1): 0.6/3.9 MBProgress (1): 0.6/3.9 MBProgress (1): 0.7/3.9 MBProgress (1): 0.7/3.9 MBProgress (1): 0.7/3.9 MBProgress (1): 0.7/3.9 MBProgress (1): 0.7/3.9 MBProgress (1): 0.7/3.9 MBProgress (1): 0.8/3.9 MBProgress (1): 0.8/3.9 MBProgress (1): 0.8/3.9 MBProgress (1): 0.8/3.9 MBProgress (1): 0.8/3.9 MBProgress (1): 0.8/3.9 MBProgress (1): 0.9/3.9 MBProgress (1): 0.9/3.9 MBProgress (1): 0.9/3.9 MBProgress (1): 0.9/3.9 MBProgress (1): 0.9/3.9 MBProgress (1): 0.9/3.9 MBProgress (1): 1.0/3.9 MBProgress (1): 1.0/3.9 MBProgress (1): 1.0/3.9 MBProgress (1): 1.0/3.9 MBProgress (1): 1.0/3.9 MBProgress (1): 1.0/3.9 MBProgress (1): 1.0/3.9 MBProgress (1): 1.1/3.9 MBProgress (1): 1.1/3.9 MBProgress (1): 1.1/3.9 MBProgress (1): 1.1/3.9 MBProgress (1): 1.1/3.9 MBProgress (1): 1.1/3.9 MBProgress (1): 1.2/3.9 MBProgress (1): 1.2/3.9 MBProgress (1): 1.2/3.9 MBProgress (1): 1.2/3.9 MBProgress (1): 1.2/3.9 MBProgress (1): 1.2/3.9 MBProgress (1): 1.3/3.9 MBProgress (1): 1.3/3.9 MBProgress (1): 1.3/3.9 MBProgress (1): 1.3/3.9 MBProgress (1): 1.3/3.9 MBProgress (1): 1.3/3.9 MBProgress (1): 1.4/3.9 MBProgress (1): 1.4/3.9 MBProgress (1): 1.4/3.9 MBProgress (1): 1.4/3.9 MBProgress (1): 1.4/3.9 MBProgress (1): 1.4/3.9 MBProgress (1): 1.5/3.9 MBProgress (1): 1.5/3.9 MBProgress (1): 1.5/3.9 MBProgress (1): 1.5/3.9 MBProgress (1): 1.5/3.9 MBProgress (1): 1.5/3.9 MBProgress (1): 1.6/3.9 MBProgress (1): 1.6/3.9 MBProgress (1): 1.6/3.9 MBProgress (1): 1.6/3.9 MBProgress (1): 1.6/3.9 MBProgress (1): 1.6/3.9 MBProgress (1): 1.6/3.9 MBProgress (1): 1.7/3.9 MBProgress (1): 1.7/3.9 MBProgress (1): 1.7/3.9 MBProgress (1): 1.7/3.9 MBProgress (1): 1.7/3.9 MBProgress (1): 1.7/3.9 MBProgress (1): 1.8/3.9 MBProgress (1): 1.8/3.9 MBProgress (1): 1.8/3.9 MBProgress (1): 1.8/3.9 MBProgress (1): 1.8/3.9 MBProgress (1): 1.8/3.9 MBProgress (1): 1.8/3.9 MBProgress (1): 1.9/3.9 MBProgress (1): 1.9/3.9 MBProgress (1): 1.9/3.9 MBProgress (1): 1.9/3.9 MBProgress (1): 1.9/3.9 MBProgress (1): 1.9/3.9 MBProgress (1): 2.0/3.9 MBProgress (1): 2.0/3.9 MBProgress (1): 2.0/3.9 MBProgress (1): 2.0/3.9 MBProgress (1): 2.0/3.9 MBProgress (1): 2.0/3.9 MBProgress (1): 2.0/3.9 MBProgress (1): 2.1/3.9 MBProgress (1): 2.1/3.9 MBProgress (1): 2.1/3.9 MBProgress (1): 2.1/3.9 MBProgress (1): 2.1/3.9 MBProgress (1): 2.1/3.9 MBProgress (1): 2.2/3.9 MBProgress (1): 2.2/3.9 MBProgress (1): 2.2/3.9 MBProgress (1): 2.2/3.9 MBProgress (1): 2.2/3.9 MBProgress (1): 2.2/3.9 MBProgress (1): 2.3/3.9 MBProgress (1): 2.3/3.9 MBProgress (1): 2.3/3.9 MBProgress (1): 2.3/3.9 MBProgress (1): 2.3/3.9 MBProgress (1): 2.3/3.9 MBProgress (1): 2.3/3.9 MBProgress (1): 2.4/3.9 MBProgress (1): 2.4/3.9 MBProgress (1): 2.4/3.9 MBProgress (1): 2.4/3.9 MBProgress (1): 2.4/3.9 MBProgress (1): 2.4/3.9 MBProgress (1): 2.5/3.9 MBProgress (1): 2.5/3.9 MBProgress (1): 2.5/3.9 MBProgress (1): 2.5/3.9 MBProgress (1): 2.5/3.9 MBProgress (1): 2.5/3.9 MBProgress (1): 2.6/3.9 MBProgress (1): 2.6/3.9 MBProgress (1): 2.6/3.9 MBProgress (1): 2.6/3.9 MBProgress (1): 2.6/3.9 MBProgress (1): 2.6/3.9 MBProgress (1): 2.6/3.9 MBProgress (1): 2.7/3.9 MBProgress (1): 2.7/3.9 MBProgress (1): 2.7/3.9 MBProgress (1): 2.7/3.9 MBProgress (1): 2.7/3.9 MBProgress (1): 2.7/3.9 MBProgress (1): 2.8/3.9 MBProgress (1): 2.8/3.9 MBProgress (1): 2.8/3.9 MBProgress (1): 2.8/3.9 MBProgress (1): 2.8/3.9 MBProgress (1): 2.8/3.9 MBProgress (1): 2.9/3.9 MBProgress (1): 2.9/3.9 MBProgress (1): 2.9/3.9 MBProgress (1): 2.9/3.9 MBProgress (1): 2.9/3.9 MBProgress (1): 2.9/3.9 MBProgress (1): 2.9/3.9 MBProgress (1): 3.0/3.9 MBProgress (1): 3.0/3.9 MBProgress (1): 3.0/3.9 MBProgress (1): 3.0/3.9 MBProgress (1): 3.0/3.9 MBProgress (1): 3.0/3.9 MBProgress (1): 3.0/3.9 MBProgress (1): 3.1/3.9 MBProgress (1): 3.1/3.9 MBProgress (1): 3.1/3.9 MBProgress (1): 3.1/3.9 MBProgress (1): 3.1/3.9 MBProgress (1): 3.1/3.9 MBProgress (1): 3.2/3.9 MBProgress (1): 3.2/3.9 MBProgress (1): 3.2/3.9 MBProgress (1): 3.2/3.9 MBProgress (1): 3.2/3.9 MBProgress (1): 3.2/3.9 MBProgress (1): 3.3/3.9 MBProgress (1): 3.3/3.9 MBProgress (1): 3.3/3.9 MBProgress (1): 3.3/3.9 MBProgress (1): 3.3/3.9 MBProgress (1): 3.3/3.9 MBProgress (1): 3.4/3.9 MBProgress (1): 3.4/3.9 MBProgress (1): 3.4/3.9 MBProgress (1): 3.4/3.9 MBProgress (1): 3.4/3.9 MBProgress (1): 3.4/3.9 MBProgress (1): 3.5/3.9 MBProgress (1): 3.5/3.9 MBProgress (1): 3.5/3.9 MBProgress (1): 3.5/3.9 MBProgress (1): 3.5/3.9 MBProgress (1): 3.5/3.9 MBProgress (1): 3.6/3.9 MBProgress (1): 3.6/3.9 MBProgress (1): 3.6/3.9 MBProgress (1): 3.6/3.9 MBProgress (1): 3.6/3.9 MBProgress (1): 3.6/3.9 MBProgress (1): 3.7/3.9 MBProgress (1): 3.7/3.9 MBProgress (1): 3.7/3.9 MBProgress (1): 3.7/3.9 MBProgress (1): 3.7/3.9 MBProgress (1): 3.7/3.9 MBProgress (1): 3.8/3.9 MBProgress (1): 3.8/3.9 MBProgress (1): 3.8/3.9 MBProgress (1): 3.8/3.9 MBProgress (1): 3.8/3.9 MBProgress (1): 3.8/3.9 MBProgress (1): 3.8/3.9 MBProgress (1): 3.9/3.9 MBProgress (1): 3.9/3.9 MBProgress (1): 3.9 MB                        Downloaded from central: https://repo.maven.apache.org/maven2/org/graalvm/truffle/truffle-dsl-processor/25.0.3/truffle-dsl-processor-25.0.3.jar (3.9 MB at 60 MB/s)
[INFO] Recompiling the module because of changed source code.
[INFO] Compiling 367 source files with javac [debug parameters release 21] to target/classes
[WARNING] /home/runner/work/hara/hara/core/java/target/generated-sources/annotations/hara/truffle/bytecode/HbcBytecodeRootNodeGen.java:[1301,59] java.lang.ThreadDeath in java.lang has been deprecated and marked for removal
[WARNING] /home/runner/work/hara/hara/core/java/target/generated-sources/annotations/hara/truffle/bytecode/HbcBytecodeRootNodeGen.java:[1784,63] java.lang.ThreadDeath in java.lang has been deprecated and marked for removal
[INFO] /home/runner/work/hara/hara/core/java/src/main/java/hara/kernel/Server.java: Some input files use or override a deprecated API.
[INFO] /home/runner/work/hara/hara/core/java/src/main/java/hara/kernel/Server.java: Recompile with -Xlint:deprecation for details.
[INFO] /home/runner/work/hara/hara/core/java/src/main/java/hara/lang/data/types/ObjPersistent.java: Some input files use unchecked or unsafe operations.
[INFO] /home/runner/work/hara/hara/core/java/src/main/java/hara/lang/data/types/ObjPersistent.java: Recompile with -Xlint:unchecked for details.
[INFO] 
[INFO] --- exec:3.5.0:java (compile-foundation-halc) @ hara.lang ---
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/resolver/maven-resolver-util/1.4.1/maven-resolver-util-1.4.1.pom
Progress (1): 825 BProgress (1): 2.8 kBProgress (1): 2.8 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/resolver/maven-resolver-util/1.4.1/maven-resolver-util-1.4.1.pom (2.8 kB at 165 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/resolver/maven-resolver/1.4.1/maven-resolver-1.4.1.pom
Progress (1): 774 BProgress (1): 2.2 kBProgress (1): 6.0 kBProgress (1): 9.6 kBProgress (1): 13 kB Progress (1): 16 kBProgress (1): 18 kB                   Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/resolver/maven-resolver/1.4.1/maven-resolver-1.4.1.pom (18 kB at 1.1 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven-parent/33/maven-parent-33.pom
Progress (1): 717 BProgress (1): 1.9 kBProgress (1): 5.4 kBProgress (1): 9.9 kBProgress (1): 14 kB Progress (1): 19 kBProgress (1): 23 kBProgress (1): 26 kBProgress (1): 27 kBProgress (1): 30 kBProgress (1): 34 kBProgress (1): 36 kBProgress (1): 39 kBProgress (1): 44 kB                   Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven-parent/33/maven-parent-33.pom (44 kB at 2.8 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/resolver/maven-resolver-api/1.4.1/maven-resolver-api-1.4.1.pom
Progress (1): 843 BProgress (1): 2.6 kBProgress (1): 2.6 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/resolver/maven-resolver-api/1.4.1/maven-resolver-api-1.4.1.pom (2.6 kB at 188 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-utils/4.0.2/plexus-utils-4.0.2.pom
Progress (1): 976 BProgress (1): 4.3 kBProgress (1): 7.7 kBProgress (1): 12 kB Progress (1): 13 kB                   Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-utils/4.0.2/plexus-utils-4.0.2.pom (13 kB at 534 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-xml/3.0.1/plexus-xml-3.0.1.pom
Progress (1): 814 BProgress (1): 2.6 kBProgress (1): 3.7 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-xml/3.0.1/plexus-xml-3.0.1.pom (3.7 kB at 246 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus/18/plexus-18.pom
Progress (1): 692 BProgress (1): 2.5 kBProgress (1): 6.4 kBProgress (1): 8.9 kBProgress (1): 10 kB Progress (1): 14 kBProgress (1): 18 kBProgress (1): 21 kBProgress (1): 25 kBProgress (1): 29 kBProgress (1): 29 kB                   Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus/18/plexus-18.pom (29 kB at 1.5 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/commons/commons-exec/1.4.0/commons-exec-1.4.0.pom
Progress (1): 757 BProgress (1): 2.2 kBProgress (1): 4.0 kBProgress (1): 6.7 kBProgress (1): 9.5 kBProgress (1): 9.5 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/commons/commons-exec/1.4.0/commons-exec-1.4.0.pom (9.5 kB at 635 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/commons/commons-parent/65/commons-parent-65.pom
Progress (1): 705 BProgress (1): 1.9 kBProgress (1): 3.1 kBProgress (1): 4.7 kBProgress (1): 6.4 kBProgress (1): 8.4 kBProgress (1): 10 kB Progress (1): 13 kBProgress (1): 15 kBProgress (1): 18 kBProgress (1): 19 kBProgress (1): 23 kBProgress (1): 25 kBProgress (1): 28 kBProgress (1): 32 kBProgress (1): 32 kBProgress (1): 35 kBProgress (1): 41 kBProgress (1): 43 kBProgress (1): 46 kBProgress (1): 49 kBProgress (1): 51 kBProgress (1): 54 kBProgress (1): 58 kBProgress (1): 60 kBProgress (1): 64 kBProgress (1): 67 kBProgress (1): 71 kBProgress (1): 74 kBProgress (1): 76 kBProgress (1): 78 kB                   Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/commons/commons-parent/65/commons-parent-65.pom (78 kB at 4.6 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/junit/junit-bom/5.10.1/junit-bom-5.10.1.pom
Progress (1): 908 BProgress (1): 4.1 kBProgress (1): 5.6 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/junit/junit-bom/5.10.1/junit-bom-5.10.1.pom (5.6 kB at 471 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/ow2/asm/asm/9.7.1/asm-9.7.1.pom
Progress (1): 1.3 kBProgress (1): 2.4 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/ow2/asm/asm/9.7.1/asm-9.7.1.pom (2.4 kB at 198 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/ow2/asm/asm-commons/9.7.1/asm-commons-9.7.1.pom
Progress (1): 1.1 kBProgress (1): 2.8 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/ow2/asm/asm-commons/9.7.1/asm-commons-9.7.1.pom (2.8 kB at 200 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/ow2/asm/asm-tree/9.7.1/asm-tree-9.7.1.pom
Progress (1): 1.1 kBProgress (1): 2.6 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/ow2/asm/asm-tree/9.7.1/asm-tree-9.7.1.pom (2.6 kB at 173 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/resolver/maven-resolver-util/1.4.1/maven-resolver-util-1.4.1.jar
Progress (1): 7.7/168 kBProgress (1): 7.7/168 kBProgress (1): 24/168 kB Progress (1): 40/168 kBProgress (1): 57/168 kBProgress (1): 73/168 kBProgress (1): 90/168 kBProgress (1): 106/168 kBProgress (1): 122/168 kBProgress (1): 139/168 kBProgress (1): 155/168 kBProgress (1): 168 kB                        Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/resolver/maven-resolver-util/1.4.1/maven-resolver-util-1.4.1.jar (168 kB at 11 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/resolver/maven-resolver-api/1.4.1/maven-resolver-api-1.4.1.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-utils/4.0.2/plexus-utils-4.0.2.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-xml/3.0.1/plexus-xml-3.0.1.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/commons/commons-exec/1.4.0/commons-exec-1.4.0.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/ow2/asm/asm/9.7.1/asm-9.7.1.jar
Progress (1): 7.7/149 kBProgress (1): 16/149 kB Progress (1): 32/149 kBProgress (1): 48/149 kBProgress (1): 65/149 kBProgress (1): 81/149 kBProgress (1): 97/149 kBProgress (1): 114/149 kBProgress (1): 130/149 kBProgress (2): 130/149 kB | 7.7/94 kBProgress (2): 130/149 kB | 12/94 kB Progress (2): 147/149 kB | 12/94 kBProgress (2): 149 kB | 12/94 kB    Progress (2): 149 kB | 29/94 kBProgress (2): 149 kB | 45/94 kBProgress (2): 149 kB | 61/94 kBProgress (3): 149 kB | 61/94 kB | 7.7/193 kBProgress (3): 149 kB | 61/94 kB | 11/193 kB Progress (3): 149 kB | 78/94 kB | 11/193 kBProgress (3): 149 kB | 94/94 kB | 11/193 kBProgress (3): 149 kB | 94 kB | 11/193 kB   Progress (3): 149 kB | 94 kB | 28/193 kBProgress (3): 149 kB | 94 kB | 44/193 kBProgress (4): 149 kB | 94 kB | 44/193 kB | 7.7/126 kBProgress (4): 149 kB | 94 kB | 61/193 kB | 7.7/126 kBProgress (4): 149 kB | 94 kB | 61/193 kB | 16/126 kB                                                     Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-xml/3.0.1/plexus-xml-3.0.1.jar (94 kB at 5.2 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/ow2/asm/asm-commons/9.7.1/asm-commons-9.7.1.jar
Progress (3): 149 kB | 77/193 kB | 16/126 kBProgress (3): 149 kB | 77/193 kB | 32/126 kBProgress (3): 149 kB | 77/193 kB | 49/126 kBProgress (3): 149 kB | 93/193 kB | 49/126 kBProgress (3): 149 kB | 93/193 kB | 65/126 kBProgress (3): 149 kB | 110/193 kB | 65/126 kBProgress (3): 149 kB | 110/193 kB | 81/126 kBProgress (3): 149 kB | 126/193 kB | 81/126 kBProgress (3): 149 kB | 126/193 kB | 98/126 kBProgress (3): 149 kB | 142/193 kB | 98/126 kBProgress (3): 149 kB | 142/193 kB | 114/126 kBProgress (3): 149 kB | 159/193 kB | 114/126 kBProgress (3): 149 kB | 159/193 kB | 126 kB    Progress (3): 149 kB | 175/193 kB | 126 kBProgress (3): 149 kB | 192/193 kB | 126 kBProgress (3): 149 kB | 193 kB | 126 kB                                          Downloaded from central: https://repo.maven.apache.org/maven2/org/ow2/asm/asm/9.7.1/asm-9.7.1.jar (126 kB at 7.4 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/ow2/asm/asm-tree/9.7.1/asm-tree-9.7.1.jar
Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/resolver/maven-resolver-api/1.4.1/maven-resolver-api-1.4.1.jar (149 kB at 8.3 MB/s)
Progress (2): 193 kB | 7.7/66 kBProgress (2): 193 kB | 15/66 kB Progress (2): 193 kB | 32/66 kBProgress (2): 193 kB | 48/66 kBProgress (2): 193 kB | 65/66 kBProgress (2): 193 kB | 66 kB                               Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/commons/commons-exec/1.4.0/commons-exec-1.4.0.jar (66 kB at 3.7 MB/s)
Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-utils/4.0.2/plexus-utils-4.0.2.jar (193 kB at 10 MB/s)
Progress (1): 3.2/52 kBProgress (2): 3.2/52 kB | 3.1/73 kBProgress (2): 20/52 kB | 3.1/73 kB Progress (2): 20/52 kB | 20/73 kB Progress (2): 28/52 kB | 20/73 kBProgress (2): 28/52 kB | 36/73 kBProgress (2): 44/52 kB | 36/73 kBProgress (2): 44/52 kB | 52/73 kBProgress (2): 52 kB | 52/73 kB   Progress (2): 52 kB | 69/73 kBProgress (2): 52 kB | 73 kB                              Downloaded from central: https://repo.maven.apache.org/maven2/org/ow2/asm/asm-tree/9.7.1/asm-tree-9.7.1.jar (52 kB at 1.8 MB/s)
Downloaded from central: https://repo.maven.apache.org/maven2/org/ow2/asm/asm-commons/9.7.1/asm-commons-9.7.1.jar (73 kB at 2.5 MB/s)
Compiled std.foundation to /home/runner/work/hara/hara/core/java/target/classes/std/foundation.halc (128221 bytes)
[INFO] 
[INFO] --- resources:3.3.1:testResources (default-testResources) @ hara.lang ---
[INFO] Copying 4 resources from src/test/resources to target/test-classes
[INFO] Copying 394 resources from ../lib/src to target/test-classes
[INFO] Copying 79 resources from ../lib/src-lang to target/test-classes
[INFO] Copying 188 resources from ../lib/test to target/test-classes
[INFO] Copying 109 resources from ../lib/test-lang to target/test-classes
[INFO] 
[INFO] --- compiler:3.13.0:testCompile (default-testCompile) @ hara.lang ---
[INFO] Recompiling the module because of changed dependency.
[INFO] Compiling 191 source files with javac [debug parameters release 21] to target/test-classes
[INFO] /home/runner/work/hara/hara/core/java/src/test/java/hara/kernel/ConnTest.java: Some input files use or override a deprecated API.
[INFO] /home/runner/work/hara/hara/core/java/src/test/java/hara/kernel/ConnTest.java: Recompile with -Xlint:deprecation for details.
[INFO] /home/runner/work/hara/hara/core/java/src/test/java/hara/kernel/ListSymbols.java: Some input files use unchecked or unsafe operations.
[INFO] /home/runner/work/hara/hara/core/java/src/test/java/hara/kernel/ListSymbols.java: Recompile with -Xlint:unchecked for details.
[INFO] 
[INFO] --- surefire:3.0.0-M5:test (default-test) @ hara.lang ---
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/maven-surefire-common/3.0.0-M5/maven-surefire-common-3.0.0-M5.pom
Progress (1): 1.5 kBProgress (1): 5.8 kBProgress (1): 9.6 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/maven-surefire-common/3.0.0-M5/maven-surefire-common-3.0.0-M5.pom (9.6 kB at 603 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-api/3.0.0-M5/surefire-api-3.0.0-M5.pom
Progress (1): 873 BProgress (1): 2.7 kBProgress (1): 3.2 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-api/3.0.0-M5/surefire-api-3.0.0-M5.pom (3.2 kB at 201 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-logger-api/3.0.0-M5/surefire-logger-api-3.0.0-M5.pom
Progress (1): 827 BProgress (1): 2.4 kBProgress (1): 3.7 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-logger-api/3.0.0-M5/surefire-logger-api-3.0.0-M5.pom (3.7 kB at 231 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-shared-utils/3.0.0-M4/surefire-shared-utils-3.0.0-M4.pom
Progress (1): 1.2 kBProgress (1): 3.9 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-shared-utils/3.0.0-M4/surefire-shared-utils-3.0.0-M4.pom (3.9 kB at 244 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire/3.0.0-M4/surefire-3.0.0-M4.pom
Progress (1): 779 BProgress (1): 2.4 kBProgress (1): 3.9 kBProgress (1): 7.3 kBProgress (1): 13 kB Progress (1): 17 kBProgress (1): 19 kBProgress (1): 22 kBProgress (1): 26 kBProgress (1): 27 kB                   Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire/3.0.0-M4/surefire-3.0.0-M4.pom (27 kB at 1.2 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-extensions-api/3.0.0-M5/surefire-extensions-api-3.0.0-M5.pom
Progress (1): 830 BProgress (1): 3.2 kBProgress (1): 4.0 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-extensions-api/3.0.0-M5/surefire-extensions-api-3.0.0-M5.pom (4.0 kB at 253 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-booter/3.0.0-M5/surefire-booter-3.0.0-M5.pom
Progress (1): 827 BProgress (1): 3.1 kBProgress (1): 4.8 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-booter/3.0.0-M5/surefire-booter-3.0.0-M5.pom (4.8 kB at 303 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-extensions-spi/3.0.0-M5/surefire-extensions-spi-3.0.0-M5.pom
Progress (1): 874 BProgress (1): 1.6 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-extensions-spi/3.0.0-M5/surefire-extensions-spi-3.0.0-M5.pom (1.6 kB at 95 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven-toolchain/3.0-alpha-2/maven-toolchain-3.0-alpha-2.pom
Progress (1): 848 BProgress (1): 2.2 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven-toolchain/3.0-alpha-2/maven-toolchain-3.0-alpha-2.pom (2.2 kB at 149 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven/3.0-alpha-2/maven-3.0-alpha-2.pom
Progress (1): 738 BProgress (1): 1.9 kBProgress (1): 3.7 kBProgress (1): 7.9 kBProgress (1): 13 kB Progress (1): 18 kBProgress (1): 21 kBProgress (1): 21 kB                   Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven/3.0-alpha-2/maven-3.0-alpha-2.pom (21 kB at 1.4 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-artifact-transfer/0.11.0/maven-artifact-transfer-0.11.0.pom
Progress (1): 1.1 kBProgress (1): 3.8 kBProgress (1): 6.6 kBProgress (1): 11 kB Progress (1): 11 kB                   Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-artifact-transfer/0.11.0/maven-artifact-transfer-0.11.0.pom (11 kB at 716 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-shared-components/33/maven-shared-components-33.pom
Progress (1): 781 BProgress (1): 2.3 kBProgress (1): 4.6 kBProgress (1): 5.1 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-shared-components/33/maven-shared-components-33.pom (5.1 kB at 318 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-common-artifact-filters/3.1.0/maven-common-artifact-filters-3.1.0.pom
Progress (1): 803 BProgress (1): 2.3 kBProgress (1): 5.3 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-common-artifact-filters/3.1.0/maven-common-artifact-filters-3.1.0.pom (5.3 kB at 352 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven-plugin-api/3.0/maven-plugin-api-3.0.pom
Progress (1): 823 BProgress (1): 2.3 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven-plugin-api/3.0/maven-plugin-api-3.0.pom (2.3 kB at 127 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven/3.0/maven-3.0.pom
Progress (1): 759 BProgress (1): 2.0 kBProgress (1): 4.6 kBProgress (1): 7.8 kBProgress (1): 14 kB Progress (1): 18 kBProgress (1): 21 kBProgress (1): 22 kB                   Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven/3.0/maven-3.0.pom (22 kB at 1.4 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven-parent/15/maven-parent-15.pom
Progress (1): 743 BProgress (1): 2.0 kBProgress (1): 6.5 kBProgress (1): 11 kB Progress (1): 15 kBProgress (1): 17 kBProgress (1): 20 kBProgress (1): 24 kB                   Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven-parent/15/maven-parent-15.pom (24 kB at 1.3 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/apache/6/apache-6.pom
Progress (1): 749 BProgress (1): 2.0 kBProgress (1): 3.7 kBProgress (1): 6.9 kBProgress (1): 11 kB Progress (1): 13 kB                   Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/apache/6/apache-6.pom (13 kB at 914 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-java/1.0.5/plexus-java-1.0.5.pom
Progress (1): 1.3 kBProgress (1): 4.1 kBProgress (1): 4.6 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-java/1.0.5/plexus-java-1.0.5.pom (4.6 kB at 329 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-languages/1.0.5/plexus-languages-1.0.5.pom
Progress (1): 1.3 kBProgress (1): 3.6 kBProgress (1): 3.9 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-languages/1.0.5/plexus-languages-1.0.5.pom (3.9 kB at 230 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus/6.2/plexus-6.2.pom
Progress (1): 695 BProgress (1): 2.7 kBProgress (1): 6.4 kBProgress (1): 9.2 kBProgress (1): 11 kB Progress (1): 16 kBProgress (1): 20 kBProgress (1): 24 kB                   Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus/6.2/plexus-6.2.pom (24 kB at 1.4 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/com/thoughtworks/qdox/qdox/2.0-M9/qdox-2.0-M9.pom
Progress (1): 881 BProgress (1): 3.4 kBProgress (1): 6.7 kBProgress (1): 11 kB Progress (1): 14 kBProgress (1): 16 kB                   Downloaded from central: https://repo.maven.apache.org/maven2/com/thoughtworks/qdox/qdox/2.0-M9/qdox-2.0-M9.pom (16 kB at 990 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/maven-surefire-common/3.0.0-M5/maven-surefire-common-3.0.0-M5.jar
Progress (1): 7.7/311 kBProgress (1): 15/311 kB Progress (1): 32/311 kBProgress (1): 48/311 kBProgress (1): 65/311 kBProgress (1): 81/311 kBProgress (1): 97/311 kBProgress (1): 114/311 kBProgress (1): 130/311 kBProgress (1): 147/311 kBProgress (1): 163/311 kBProgress (1): 179/311 kBProgress (1): 196/311 kBProgress (1): 212/311 kBProgress (1): 228/311 kBProgress (1): 245/311 kBProgress (1): 261/311 kBProgress (1): 278/311 kBProgress (1): 294/311 kBProgress (1): 310/311 kBProgress (1): 311 kB                        Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/maven-surefire-common/3.0.0-M5/maven-surefire-common-3.0.0-M5.jar (311 kB at 19 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-api/3.0.0-M5/surefire-api-3.0.0-M5.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-logger-api/3.0.0-M5/surefire-logger-api-3.0.0-M5.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-extensions-api/3.0.0-M5/surefire-extensions-api-3.0.0-M5.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-booter/3.0.0-M5/surefire-booter-3.0.0-M5.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-extensions-spi/3.0.0-M5/surefire-extensions-spi-3.0.0-M5.jar
Progress (1): 7.7/144 kBProgress (1): 16/144 kB Progress (1): 24/144 kBProgress (1): 40/144 kBProgress (1): 57/144 kBProgress (1): 73/144 kBProgress (1): 90/144 kBProgress (1): 106/144 kBProgress (1): 122/144 kBProgress (1): 139/144 kBProgress (2): 139/144 kB | 7.7/106 kBProgress (2): 144 kB | 7.7/106 kB    Progress (2): 144 kB | 7.7/106 kBProgress (2): 144 kB | 24/106 kB Progress (2): 144 kB | 40/106 kB                                Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-api/3.0.0-M5/surefire-api-3.0.0-M5.jar (144 kB at 10 MB/s)
Progress (1): 57/106 kBProgress (1): 73/106 kBProgress (1): 90/106 kBProgress (1): 106 kB                       Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-booter/3.0.0-M5/surefire-booter-3.0.0-M5.jar (106 kB at 7.6 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven-toolchain/3.0-alpha-2/maven-toolchain-3.0-alpha-2.jar
Progress (1): 7.7/14 kBProgress (1): 11/14 kB Progress (1): 14 kB                      Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-logger-api/3.0.0-M5/surefire-logger-api-3.0.0-M5.jar (14 kB at 910 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-artifact-transfer/0.11.0/maven-artifact-transfer-0.11.0.jar
Progress (1): 7.7/7.9 kBProgress (1): 7.9 kB    Progress (2): 7.9 kB | 7.7/24 kBProgress (2): 7.9 kB | 15/24 kB Progress (2): 7.9 kB | 24 kB                               Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-extensions-spi/3.0.0-M5/surefire-extensions-spi-3.0.0-M5.jar (7.9 kB at 467 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-common-artifact-filters/3.1.0/maven-common-artifact-filters-3.1.0.jar
Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-extensions-api/3.0.0-M5/surefire-extensions-api-3.0.0-M5.jar (24 kB at 1.4 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven-plugin-api/3.0/maven-plugin-api-3.0.jar
Downloading from central: https://repo.maven.apache.org/maven2/commons-codec/commons-codec/1.11/commons-codec-1.11.jar
Progress (1): 3.2/128 kBProgress (1): 20/128 kB Progress (1): 36/128 kBProgress (1): 52/128 kBProgress (2): 52/128 kB | 7.7/61 kBProgress (2): 52/128 kB | 11/61 kB Progress (2): 69/128 kB | 11/61 kBProgress (2): 85/128 kB | 11/61 kBProgress (2): 85/128 kB | 28/61 kBProgress (2): 101/128 kB | 28/61 kBProgress (2): 101/128 kB | 44/61 kBProgress (2): 118/128 kB | 44/61 kBProgress (2): 118/128 kB | 61/61 kBProgress (2): 128 kB | 61/61 kB    Progress (2): 128 kB | 61 kB                               Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-common-artifact-filters/3.1.0/maven-common-artifact-filters-3.1.0.jar (61 kB at 2.2 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-java/1.0.5/plexus-java-1.0.5.jar
Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/shared/maven-artifact-transfer/0.11.0/maven-artifact-transfer-0.11.0.jar (128 kB at 4.6 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/com/thoughtworks/qdox/qdox/2.0-M9/qdox-2.0-M9.jar
Progress (1): 7.7/36 kBProgress (1): 11/36 kB Progress (1): 28/36 kBProgress (2): 28/36 kB | 7.7/335 kBProgress (2): 36 kB | 7.7/335 kB   Progress (2): 36 kB | 16/335 kB Progress (3): 36 kB | 16/335 kB | 7.7/49 kBProgress (3): 36 kB | 16/335 kB | 15/49 kB Progress (3): 36 kB | 24/335 kB | 15/49 kBProgress (3): 36 kB | 24/335 kB | 32/49 kBProgress (3): 36 kB | 24/335 kB | 48/49 kBProgress (3): 36 kB | 24/335 kB | 49 kB   Progress (3): 36 kB | 40/335 kB | 49 kB                                       Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven-plugin-api/3.0/maven-plugin-api-3.0.jar (49 kB at 1.6 MB/s)
Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/maven-toolchain/3.0-alpha-2/maven-toolchain-3.0-alpha-2.jar (36 kB at 1.2 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-shared-utils/3.0.0-M4/surefire-shared-utils-3.0.0-M4.jar
Progress (1): 57/335 kBProgress (1): 73/335 kBProgress (1): 90/335 kBProgress (1): 106/335 kBProgress (1): 115/335 kBProgress (1): 131/335 kBProgress (1): 147/335 kBProgress (1): 164/335 kBProgress (1): 180/335 kBProgress (1): 197/335 kBProgress (1): 213/335 kBProgress (1): 229/335 kBProgress (1): 246/335 kBProgress (1): 262/335 kBProgress (1): 279/335 kBProgress (1): 295/335 kBProgress (1): 311/335 kBProgress (1): 328/335 kBProgress (1): 335 kB                        Downloaded from central: https://repo.maven.apache.org/maven2/commons-codec/commons-codec/1.11/commons-codec-1.11.jar (335 kB at 10 MB/s)
Progress (1): 7.7/317 kBProgress (1): 7.7/317 kBProgress (1): 24/317 kB Progress (1): 40/317 kBProgress (1): 57/317 kBProgress (1): 73/317 kBProgress (1): 90/317 kBProgress (1): 106/317 kBProgress (2): 106/317 kB | 7.7/52 kBProgress (2): 106/317 kB | 16/52 kB Progress (2): 106/317 kB | 24/52 kBProgress (2): 106/317 kB | 40/52 kBProgress (2): 122/317 kB | 40/52 kBProgress (2): 122/317 kB | 52 kB   Progress (2): 139/317 kB | 52 kB                                Downloaded from central: https://repo.maven.apache.org/maven2/org/codehaus/plexus/plexus-java/1.0.5/plexus-java-1.0.5.jar (52 kB at 1.2 MB/s)
Progress (2): 139/317 kB | 0/1.9 MBProgress (2): 139/317 kB | 0/1.9 MBProgress (2): 139/317 kB | 0/1.9 MBProgress (2): 139/317 kB | 0/1.9 MBProgress (2): 155/317 kB | 0/1.9 MBProgress (2): 172/317 kB | 0/1.9 MBProgress (2): 188/317 kB | 0/1.9 MBProgress (2): 197/317 kB | 0/1.9 MBProgress (2): 213/317 kB | 0/1.9 MBProgress (2): 229/317 kB | 0/1.9 MBProgress (2): 246/317 kB | 0/1.9 MBProgress (2): 262/317 kB | 0/1.9 MBProgress (2): 279/317 kB | 0/1.9 MBProgress (2): 295/317 kB | 0/1.9 MBProgress (2): 311/317 kB | 0/1.9 MBProgress (2): 317 kB | 0/1.9 MB                                   Downloaded from central: https://repo.maven.apache.org/maven2/com/thoughtworks/qdox/qdox/2.0-M9/qdox-2.0-M9.jar (317 kB at 7.2 MB/s)
Progress (1): 0/1.9 MBProgress (1): 0.1/1.9 MBProgress (1): 0.1/1.9 MBProgress (1): 0.1/1.9 MBProgress (1): 0.1/1.9 MBProgress (1): 0.1/1.9 MBProgress (1): 0.1/1.9 MBProgress (1): 0.2/1.9 MBProgress (1): 0.2/1.9 MBProgress (1): 0.2/1.9 MBProgress (1): 0.2/1.9 MBProgress (1): 0.2/1.9 MBProgress (1): 0.2/1.9 MBProgress (1): 0.3/1.9 MBProgress (1): 0.3/1.9 MBProgress (1): 0.3/1.9 MBProgress (1): 0.3/1.9 MBProgress (1): 0.3/1.9 MBProgress (1): 0.3/1.9 MBProgress (1): 0.4/1.9 MBProgress (1): 0.4/1.9 MBProgress (1): 0.4/1.9 MBProgress (1): 0.4/1.9 MBProgress (1): 0.4/1.9 MBProgress (1): 0.4/1.9 MBProgress (1): 0.5/1.9 MBProgress (1): 0.5/1.9 MBProgress (1): 0.5/1.9 MBProgress (1): 0.5/1.9 MBProgress (1): 0.5/1.9 MBProgress (1): 0.5/1.9 MBProgress (1): 0.6/1.9 MBProgress (1): 0.6/1.9 MBProgress (1): 0.6/1.9 MBProgress (1): 0.6/1.9 MBProgress (1): 0.6/1.9 MBProgress (1): 0.6/1.9 MBProgress (1): 0.7/1.9 MBProgress (1): 0.7/1.9 MBProgress (1): 0.7/1.9 MBProgress (1): 0.7/1.9 MBProgress (1): 0.7/1.9 MBProgress (1): 0.7/1.9 MBProgress (1): 0.8/1.9 MBProgress (1): 0.8/1.9 MBProgress (1): 0.8/1.9 MBProgress (1): 0.8/1.9 MBProgress (1): 0.8/1.9 MBProgress (1): 0.8/1.9 MBProgress (1): 0.9/1.9 MBProgress (1): 0.9/1.9 MBProgress (1): 0.9/1.9 MBProgress (1): 0.9/1.9 MBProgress (1): 0.9/1.9 MBProgress (1): 0.9/1.9 MBProgress (1): 0.9/1.9 MBProgress (1): 1.0/1.9 MBProgress (1): 1.0/1.9 MBProgress (1): 1.0/1.9 MBProgress (1): 1.0/1.9 MBProgress (1): 1.0/1.9 MBProgress (1): 1.0/1.9 MBProgress (1): 1.1/1.9 MBProgress (1): 1.1/1.9 MBProgress (1): 1.1/1.9 MBProgress (1): 1.1/1.9 MBProgress (1): 1.1/1.9 MBProgress (1): 1.1/1.9 MBProgress (1): 1.2/1.9 MBProgress (1): 1.2/1.9 MBProgress (1): 1.2/1.9 MBProgress (1): 1.2/1.9 MBProgress (1): 1.2/1.9 MBProgress (1): 1.2/1.9 MBProgress (1): 1.3/1.9 MBProgress (1): 1.3/1.9 MBProgress (1): 1.3/1.9 MBProgress (1): 1.3/1.9 MBProgress (1): 1.3/1.9 MBProgress (1): 1.3/1.9 MBProgress (1): 1.3/1.9 MBProgress (1): 1.4/1.9 MBProgress (1): 1.4/1.9 MBProgress (1): 1.4/1.9 MBProgress (1): 1.4/1.9 MBProgress (1): 1.4/1.9 MBProgress (1): 1.4/1.9 MBProgress (1): 1.4/1.9 MBProgress (1): 1.5/1.9 MBProgress (1): 1.5/1.9 MBProgress (1): 1.5/1.9 MBProgress (1): 1.5/1.9 MBProgress (1): 1.5/1.9 MBProgress (1): 1.5/1.9 MBProgress (1): 1.5/1.9 MBProgress (1): 1.6/1.9 MBProgress (1): 1.6/1.9 MBProgress (1): 1.6/1.9 MBProgress (1): 1.6/1.9 MBProgress (1): 1.6/1.9 MBProgress (1): 1.6/1.9 MBProgress (1): 1.6/1.9 MBProgress (1): 1.7/1.9 MBProgress (1): 1.7/1.9 MBProgress (1): 1.7/1.9 MBProgress (1): 1.7/1.9 MBProgress (1): 1.7/1.9 MBProgress (1): 1.7/1.9 MBProgress (1): 1.8/1.9 MBProgress (1): 1.8/1.9 MBProgress (1): 1.8/1.9 MBProgress (1): 1.8/1.9 MBProgress (1): 1.8/1.9 MBProgress (1): 1.8/1.9 MBProgress (1): 1.8/1.9 MBProgress (1): 1.9/1.9 MBProgress (1): 1.9 MB                        Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-shared-utils/3.0.0-M4/surefire-shared-utils-3.0.0-M4.jar (1.9 MB at 32 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-junit4/3.0.0-M5/surefire-junit4-3.0.0-M5.jar
Progress (1): 7.7/17 kBProgress (1): 7.7/17 kBProgress (1): 17 kB                       Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-junit4/3.0.0-M5/surefire-junit4-3.0.0-M5.jar (17 kB at 1.1 MB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-junit4/3.0.0-M5/surefire-junit4-3.0.0-M5.pom
Progress (1): 873 BProgress (1): 2.7 kBProgress (1): 2.9 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-junit4/3.0.0-M5/surefire-junit4-3.0.0-M5.pom (2.9 kB at 205 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-providers/3.0.0-M5/surefire-providers-3.0.0-M5.pom
Progress (1): 844 BProgress (1): 2.5 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/surefire-providers/3.0.0-M5/surefire-providers-3.0.0-M5.pom (2.5 kB at 158 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/junit/junit/4.0/junit-4.0.pom
Progress (1): 210 B                   Downloaded from central: https://repo.maven.apache.org/maven2/junit/junit/4.0/junit-4.0.pom (210 B at 12 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-junit4/3.0.0-M5/common-junit4-3.0.0-M5.pom
Progress (1): 859 BProgress (1): 2.8 kBProgress (1): 2.9 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-junit4/3.0.0-M5/common-junit4-3.0.0-M5.pom (2.9 kB at 155 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-junit3/3.0.0-M5/common-junit3-3.0.0-M5.pom
Progress (1): 873 BProgress (1): 2.6 kBProgress (1): 2.6 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-junit3/3.0.0-M5/common-junit3-3.0.0-M5.pom (2.6 kB at 163 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-java5/3.0.0-M5/common-java5-3.0.0-M5.pom
Progress (1): 872 BProgress (1): 2.6 kBProgress (1): 2.6 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-java5/3.0.0-M5/common-java5-3.0.0-M5.pom (2.6 kB at 175 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/hamcrest/hamcrest-library/1.3/hamcrest-library-1.3.pom
Progress (1): 820 B                   Downloaded from central: https://repo.maven.apache.org/maven2/org/hamcrest/hamcrest-library/1.3/hamcrest-library-1.3.pom (820 B at 59 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/easytesting/fest-assert/1.4/fest-assert-1.4.pom
Progress (1): 1.3 kBProgress (1): 2.4 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/easytesting/fest-assert/1.4/fest-assert-1.4.pom (2.4 kB at 158 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/easytesting/fest/1.0.8/fest-1.0.8.pom
Progress (1): 1.2 kBProgress (1): 4.0 kBProgress (1): 7.2 kBProgress (1): 11 kB Progress (1): 12 kB                   Downloaded from central: https://repo.maven.apache.org/maven2/org/easytesting/fest/1.0.8/fest-1.0.8.pom (12 kB at 756 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/easytesting/fest-util/1.1.6/fest-util-1.1.6.pom
Progress (1): 1.3 kBProgress (1): 1.6 kB                    Downloaded from central: https://repo.maven.apache.org/maven2/org/easytesting/fest-util/1.1.6/fest-util-1.1.6.pom (1.6 kB at 97 kB/s)
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-junit3/3.0.0-M5/common-junit3-3.0.0-M5.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-junit4/3.0.0-M5/common-junit4-3.0.0-M5.jar
Downloading from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-java5/3.0.0-M5/common-java5-3.0.0-M5.jar
Progress (1): 7.7/25 kBProgress (1): 11/25 kB Progress (1): 25 kB                      Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-junit4/3.0.0-M5/common-junit4-3.0.0-M5.jar (25 kB at 1.8 MB/s)
Progress (1): 3.2/12 kBProgress (1): 12 kB                       Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-junit3/3.0.0-M5/common-junit3-3.0.0-M5.jar (12 kB at 782 kB/s)
Progress (1): 7.7/16 kBProgress (1): 16 kB                       Downloaded from central: https://repo.maven.apache.org/maven2/org/apache/maven/surefire/common-java5/3.0.0-M5/common-java5-3.0.0-M5.jar (16 kB at 1.1 MB/s)
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
[ERROR] Tests run: 5, Failures: 1, Errors: 0, Skipped: 0, Time elapsed: 2.079 s <<< FAILURE! - in hara.truffle.ToolVmLibraryTest
[ERROR] hara.truffle.ToolVmLibraryTest.providerReportsOnlyTheImplementedTransformation  Time elapsed: 0.012 s  <<< FAILURE!
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
[INFO] Total time:  20.643 s
[INFO] Finished at: 2026-08-18T10:11:35Z
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
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

[1m[33mwarning[0m[1m: unused imports: `PathBuf` and `Path`[0m
   [1m[94m--> [0msrc/core/provider.rs:220:17
    [1m[94m|[0m
[1m[94m220[0m [1m[94m|[0m use std::path::{Path, PathBuf};
    [1m[94m|[0m                 [1m[33m^^^^[0m  [1m[33m^^^^^^^[0m

[1m[33mwarning[0m[1m: unused import: `std::rc::Rc`[0m
 [1m[94m--> [0msrc/work/guest.rs:6:5
  [1m[94m|[0m
[1m[94m6[0m [1m[94m|[0m use std::rc::Rc;
  [1m[94m|[0m     [1m[33m^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: use of deprecated method `sha2::digest::generic_array::GenericArray::<T, N>::as_slice`: please upgrade to generic-array 1.x[0m
   [1m[94m--> [0msrc/kernel/halc.rs:124:20
    [1m[94m|[0m
[1m[94m124[0m [1m[94m|[0m     if actual_hash.as_slice() != expected_hash.as_slice() {
    [1m[94m|[0m                    [1m[33m^^^^^^^^[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: `#[warn(deprecated)]` on by default

[1m[33mwarning[0m[1m: use of deprecated method `sha2::digest::generic_array::GenericArray::<T, N>::as_slice`: please upgrade to generic-array 1.x[0m
   [1m[94m--> [0msrc/kernel/halc.rs:448:65
    [1m[94m|[0m
[1m[94m448[0m [1m[94m|[0m     payload.extend_from_slice(Sha256::digest(source.as_bytes()).as_slice());
    [1m[94m|[0m                                                                 [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: use of deprecated method `sha2::digest::generic_array::GenericArray::<T, N>::as_slice`: please upgrade to generic-array 1.x[0m
   [1m[94m--> [0msrc/kernel/halc.rs:458:57
    [1m[94m|[0m
[1m[94m458[0m [1m[94m|[0m     artifact.extend_from_slice(Sha256::digest(&payload).as_slice());
    [1m[94m|[0m                                                         [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: use of deprecated method `sha2::digest::generic_array::GenericArray::<T, N>::as_slice`: please upgrade to generic-array 1.x[0m
  [1m[94m--> [0msrc/kernel/halc_source_trace/evidence.rs:97:31
   [1m[94m|[0m
[1m[94m97[0m [1m[94m|[0m     hex(Sha256::digest(bytes).as_slice())
   [1m[94m|[0m                               [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: use of deprecated method `sha2::digest::generic_array::GenericArray::<T, N>::as_slice`: please upgrade to generic-array 1.x[0m
  [1m[94m--> [0msrc/vm/artifact.rs:69:32
   [1m[94m|[0m
[1m[94m69[0m [1m[94m|[0m     if Sha256::digest(payload).as_slice() != &bytes[payload_end..] {
   [1m[94m|[0m                                [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: use of deprecated method `sha2::digest::generic_array::GenericArray::<T, N>::as_slice`: please upgrade to generic-array 1.x[0m
   [1m[94m--> [0msrc/vm/bundle.rs:313:32
    [1m[94m|[0m
[1m[94m313[0m [1m[94m|[0m     if Sha256::digest(payload).as_slice() != &bytes[4..36] {
    [1m[94m|[0m                                [1m[33m^^^^^^^^[0m

[1m[33mwarning[0m[1m: unused import: `crate::lang::protocol::IComponent`[0m
 [1m[94m--> [0msrc/work/guest.rs:3:5
  [1m[94m|[0m
[1m[94m3[0m [1m[94m|[0m use crate::lang::protocol::IComponent;
  [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
   [1m[94m--> [0msrc/core/value.rs:881:24
    [1m[94m|[0m
[1m[94m881[0m [1m[94m|[0m             .and_then(|mut opt| {
    [1m[94m|[0m                        [1m[94m----[0m[1m[33m^^^[0m
    [1m[94m|[0m                        [1m[94m|[0m
    [1m[94m|[0m                        [1m[94mhelp: remove this `mut`[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mnote[0m: `#[warn(unused_mut)]` (part of `#[warn(unused)]`) on by default

[1m[33mwarning[0m[1m: unused variable: `env`[0m
    [1m[94m--> [0msrc/core/value.rs:1150:5
     [1m[94m|[0m
[1m[94m1150[0m [1m[94m|[0m     env: &mut HashMap<String, Value>,
     [1m[94m|[0m     [1m[33m^^^[0m [1m[33mhelp: if this is intentional, prefix it with an underscore: `_env`[0m
     [1m[94m|[0m
     [1m[94m= [0m[1mnote[0m: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

[1m[33mwarning[0m[1m: unused variable: `value`[0m
   [1m[94m--> [0msrc/core/async_value.rs:289:24
    [1m[94m|[0m
[1m[94m289[0m [1m[94m|[0m                     Ok(value) if matches!(*coroutine.state.borrow(), CoroutineState::Dead) => {
    [1m[94m|[0m                        [1m[33m^^^^^[0m [1m[33mhelp: if this is intentional, prefix it with an underscore: `_value`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
    [1m[94m--> [0msrc/core/operation.rs:1037:9
     [1m[94m|[0m
[1m[94m1037[0m [1m[94m|[0m     let mut state = IteratorState::generated(IteratorGenerator::Prepend(Some(head), source));
     [1m[94m|[0m         [1m[94m----[0m[1m[33m^^^^^[0m
     [1m[94m|[0m         [1m[94m|[0m
     [1m[94m|[0m         [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
  [1m[94m--> [0msrc/vm/fiber.rs:29:13
   [1m[94m|[0m
[1m[94m29[0m [1m[94m|[0m         let mut machine = Machine::entry(program);
   [1m[94m|[0m             [1m[94m----[0m[1m[33m^^^^^^^[0m
   [1m[94m|[0m             [1m[94m|[0m
   [1m[94m|[0m             [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
  [1m[94m--> [0msrc/vm/fiber.rs:51:13
   [1m[94m|[0m
[1m[94m51[0m [1m[94m|[0m         let mut machine = Machine::call(program, prototype, arguments, captures);
   [1m[94m|[0m             [1m[94m----[0m[1m[33m^^^^^^^[0m
   [1m[94m|[0m             [1m[94m|[0m
   [1m[94m|[0m             [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
  [1m[94m--> [0msrc/vm/machine/async_runtime.rs:36:9
   [1m[94m|[0m
[1m[94m36[0m [1m[94m|[0m         mut machine: Machine,
   [1m[94m|[0m         [1m[94m----[0m[1m[33m^^^^^^^[0m
   [1m[94m|[0m         [1m[94m|[0m
   [1m[94m|[0m         [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: variable does not need to be mutable[0m
   [1m[94m--> [0msrc/vm/machine.rs:551:25
    [1m[94m|[0m
[1m[94m551[0m [1m[94m|[0m                     let mut next_ip = ip;
    [1m[94m|[0m                         [1m[94m----[0m[1m[33m^^^^^^^[0m
    [1m[94m|[0m                         [1m[94m|[0m
    [1m[94m|[0m                         [1m[94mhelp: remove this `mut`[0m

[1m[33mwarning[0m[1m: associated function `new` is never used[0m
  [1m[94m--> [0msrc/runtime/session.rs:67:8
   [1m[94m|[0m
[1m[94m66[0m [1m[94m|[0m impl Session {
   [1m[94m|[0m [1m[94m------------[0m [1m[94massociated function in this implementation[0m
[1m[94m67[0m [1m[94m|[0m     fn new(name: &str, runtime: Runtime) -> Self {
   [1m[94m|[0m        [1m[33m^^^[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

[1m[33mwarning[0m[1m: function `map_entries_for_test` is never used[0m
   [1m[94m--> [0msrc/cli_app/manifest.rs:139:15
    [1m[94m|[0m
[1m[94m139[0m [1m[94m|[0m pub(super) fn map_entries_for_test(form: &Form) -> Result<&[(Form, Form)], String> {
    [1m[94m|[0m               [1m[33m^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `map_value_for_test` is never used[0m
   [1m[94m--> [0msrc/cli_app/manifest.rs:143:15
    [1m[94m|[0m
[1m[94m143[0m [1m[94m|[0m pub(super) fn map_value_for_test<'a>(entries: &'a [(Form, Form)], key: &str) -> Option<&'a Form> {
    [1m[94m|[0m               [1m[33m^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `parse` is never used[0m
   [1m[94m--> [0msrc/core/async_value.rs:395:4
    [1m[94m|[0m
[1m[94m395[0m [1m[94m|[0m fn parse(source: &str) -> Result<Form, String> {
    [1m[94m|[0m    [1m[33m^^^^^[0m

[1m[33mwarning[0m[1m: function `iterator_map_spread` is never used[0m
    [1m[94m--> [0msrc/core/operation.rs:1062:4
     [1m[94m|[0m
[1m[94m1062[0m [1m[94m|[0m fn iterator_map_spread(function: Rc<Function>, value: Value) -> Result<Value, String> {
     [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `unsupported_handoff_evidence` is never used[0m
  [1m[94m--> [0msrc/kernel/halc_bytecode_trace.rs:97:4
   [1m[94m|[0m
[1m[94m97[0m [1m[94m|[0m fn unsupported_handoff_evidence(
   [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: associated function `load` is never used[0m
  [1m[94m--> [0msrc/native_extension.rs:21:12
   [1m[94m|[0m
[1m[94m20[0m [1m[94m|[0m impl ExtensionPackage {
   [1m[94m|[0m [1m[94m---------------------[0m [1m[94massociated function in this implementation[0m
[1m[94m21[0m [1m[94m|[0m     pub fn load(root: &Path) -> Result<Self, String> {
   [1m[94m|[0m            [1m[33m^^^^[0m

[1m[33mwarning[0m[1m: function `packages_in_project` is never used[0m
   [1m[94m--> [0msrc/native_extension.rs:148:4
    [1m[94m|[0m
[1m[94m148[0m [1m[94m|[0m fn packages_in_project(root: &Path) -> Result<Vec<ExtensionPackage>, String> {
    [1m[94m|[0m    [1m[33m^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: method `to_i64_exact` is never used[0m
   [1m[94m--> [0msrc/numeric.rs:325:19
    [1m[94m|[0m
[1m[94m 64[0m [1m[94m|[0m impl ExactDecimal {
    [1m[94m|[0m [1m[94m-----------------[0m [1m[94mmethod in this implementation[0m
[1m[94m...[0m
[1m[94m325[0m [1m[94m|[0m     pub(crate) fn to_i64_exact(&self) -> Option<i64> {
    [1m[94m|[0m                   [1m[33m^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `canonical_decimal` is never used[0m
   [1m[94m--> [0msrc/numeric.rs:341:15
    [1m[94m|[0m
[1m[94m341[0m [1m[94m|[0m pub(crate) fn canonical_decimal(value: &str) -> Result<String, String> {
    [1m[94m|[0m               [1m[33m^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `numeric_increment` is never used[0m
   [1m[94m--> [0msrc/numeric.rs:652:15
    [1m[94m|[0m
[1m[94m652[0m [1m[94m|[0m pub(crate) fn numeric_increment(value: &Value, delta: i64) -> Result<Value, String> {
    [1m[94m|[0m               [1m[33m^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `to_i32_exact` is never used[0m
   [1m[94m--> [0msrc/numeric.rs:715:15
    [1m[94m|[0m
[1m[94m715[0m [1m[94m|[0m pub(crate) fn to_i32_exact(value: &Value) -> Result<i32, String> {
    [1m[94m|[0m               [1m[33m^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `to_u32_exact` is never used[0m
   [1m[94m--> [0msrc/numeric.rs:727:15
    [1m[94m|[0m
[1m[94m727[0m [1m[94m|[0m pub(crate) fn to_u32_exact(value: &Value) -> Result<u32, String> {
    [1m[94m|[0m               [1m[33m^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: function `invoke_zero_arity` is never used[0m
  [1m[94m--> [0msrc/project/production/bundle/load.rs:16:15
   [1m[94m|[0m
[1m[94m16[0m [1m[94m|[0m pub(super) fn invoke_zero_arity(runtime: &Runtime, symbol: &str) -> Result<Value, String> {
   [1m[94m|[0m               [1m[33m^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: field `plan` is never read[0m
 [1m[94m--> [0msrc/project/production/bundle/model.rs:6:37
  [1m[94m|[0m
[1m[94m5[0m [1m[94m|[0m pub(in crate::task::production) struct ProductionBuild {
  [1m[94m|[0m                                        [1m[94m---------------[0m [1m[94mfield in this struct[0m
[1m[94m6[0m [1m[94m|[0m     pub(in crate::task::production) plan: BuildPlan,
  [1m[94m|[0m                                     [1m[33m^^^^[0m
  [1m[94m|[0m
  [1m[94m= [0m[1mnote[0m: `ProductionBuild` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

[1m[33mwarning[0m[1m: method `locals` is never used[0m
   [1m[94m--> [0msrc/vm/frame.rs:124:19
    [1m[94m|[0m
[1m[94m 11[0m [1m[94m|[0m impl Frame {
    [1m[94m|[0m [1m[94m----------[0m [1m[94mmethod in this implementation[0m
[1m[94m...[0m
[1m[94m124[0m [1m[94m|[0m     pub(crate) fn locals(&self) -> &[VmSlot] {
    [1m[94m|[0m                   [1m[33m^^^^^^[0m

[1m[33mwarning[0m: `hara-wasm` (lib) generated 34 warnings (run `cargo fix --lib -p hara-wasm` to apply 11 suggestions)
[1m[92m   Compiling[0m hara-wasm v0.1.6 (/home/runner/work/hara/hara/core/rust)
[1m[92m    Finished[0m `dev` profile [unoptimized + debuginfo] target(s) in 1.11s
[1m[92m     Running[0m `core/rust/target/debug/hara-test --root . core/lib/test/tool/vm_test.hal core/lib/test/tool/vm_provider_test.hal`
hara-test: ./core/lib/test/tool/vm_provider_test.hal: cannot read /home/runner/work/hara/hara/project.edn: No such file or directory (os error 2)
```

## bounds

```text
src/work.rs grew to 977 lines; recorded legacy maximum is 920
```
