use super::*;

const JVM_SHA: &str = "sha256:c002b77f9f7b3b1b74771be2e5c75da33c6911c6f2d10689f69242cb184d9b3b";
const WASM_SHA: &str = "sha256:336154bf67f765f8f75d16a0accee61b5ee5f6a75b2a2905703df913bd550f3e";

fn requirements(
    target: &str,
    abi: &str,
    capabilities: &[&str],
    host_calls: &[&str],
) -> PackageRuntimeRequirements {
    PackageRuntimeRequirements {
        supported_targets: [target.to_owned()].into_iter().collect(),
        supported_abis: [abi.to_owned()].into_iter().collect(),
        available_capabilities: capabilities
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
        allowed_host_calls: host_calls.iter().map(|value| (*value).to_owned()).collect(),
    }
}

fn dual_runtime_manifest() -> String {
    format!(
        r#"{{:harp/format "0.0.0-alpha"
 :package {{:identity "hara:example/provider"
       :version "1.0.0"
       :provenance {{:repository "https://github.com/example/provider"
                    :commit "0123456789abcdef0123456789abcdef01234567"}}}}
 :files {{"artifacts/provider.jar" {{:sha256 "{JVM_SHA}" :size 4}}
      "artifacts/provider.hta" {{:sha256 "{WASM_SHA}" :size 4}}}}
 :variants
 {{:jvm
   {{:variant/artifact
 {{:artifact/type :jar
  :artifact/path "artifacts/provider.jar"
  :artifact/sha256 "{JVM_SHA}"
  :artifact/target "java-21"
  :artifact/abi "hara.provider.jvm.v1"
  :artifact/entry-point "example.provider.HaraProvider"}}
:variant/required-capabilities #{{:db/connect}}
:variant/host-calls #{{}}
:variant/exports #{{:provider/open :provider/close}}
:variant/dependencies {{:maven {{org.example/runtime {{:version "1.0.0"}}}}}}
:variant/lifecycle
{{:lifecycle/load :idempotent
 :lifecycle/close :idempotent
 :lifecycle/session-isolation true
 :lifecycle/async false
 :lifecycle/cancellation false}}}}
  :wasm
   {{:variant/artifact
 {{:artifact/type :hta
  :artifact/path "artifacts/provider.hta"
  :artifact/sha256 "{WASM_SHA}"
  :artifact/target "wasm32-wasi-preview1"
  :artifact/abi "hta.v1"
  :artifact/entry-point "provider_init"}}
:variant/required-capabilities #{{:db/connect}}
:variant/host-calls #{{:db/socket}}
:variant/exports #{{:provider/open :provider/cancel :provider/close}}
:variant/dependencies {{}}
:variant/lifecycle
{{:lifecycle/load :idempotent
 :lifecycle/close :idempotent
 :lifecycle/session-isolation true
 :lifecycle/async true
 :lifecycle/cancellation true}}}}}}
 :descriptor {{:operations [:provider/open :provider/close]}}}}"#
    )
}

#[test]
fn selects_exact_jvm_and_wasm_variants() {
    let manifest = PackageManifest::parse(&dual_runtime_manifest()).unwrap();
    let jvm = manifest
        .select_variant(
            PackageRuntime::Jvm,
            &requirements("java-21", "hara.provider.jvm.v1", &["db/connect"], &[]),
        )
        .unwrap();
    let PackageSelection::Variant(jvm) = &jvm else {
        panic!("expected JVM variant");
    };
    assert_eq!(jvm.artifact.artifact_type, PackageArtifactType::Jar);
    assert_eq!(jvm.artifact.entry_point, "example.provider.HaraProvider");
    manifest
        .verify_artifact_bytes(&PackageSelection::Variant(jvm.clone()), b"jvm!")
        .unwrap();

    let wasm = manifest
        .select_variant(
            PackageRuntime::Wasm,
            &requirements(
                "wasm32-wasi-preview1",
                "hta.v1",
                &["db/connect"],
                &["db/socket"],
            ),
        )
        .unwrap();
    let PackageSelection::Variant(wasm) = &wasm else {
        panic!("expected Wasm variant");
    };
    assert_eq!(wasm.artifact.artifact_type, PackageArtifactType::Hta);
    assert!(wasm
        .lifecycle
        .as_ref()
        .is_some_and(|lifecycle| lifecycle.asynchronous && lifecycle.cancellation));
    manifest
        .verify_artifact_bytes(&PackageSelection::Variant(wasm.clone()), b"wasm")
        .unwrap();
}

#[test]
fn preserves_portable_packages_and_forbids_runtime_fallback() {
    let portable = PackageManifest::parse(
        r#"{:harp/format "0.0.0-alpha"
             :package {:identity "hara:example/portable" :version "1.0.0"}
             :files {"src/example/core.hal"
                     {:sha256 "sha256:b8ba2ec7e90713c1043778164af3250820943c2165c9f19fa29987e016aae5dd"
                      :size 4}}}"#,
    )
    .unwrap();
    assert_eq!(
        portable
            .select_variant(PackageRuntime::Wasm, &PackageRuntimeRequirements::default())
            .unwrap(),
        PackageSelection::Portable
    );

    let jvm_only = PackageManifest::parse(&format!(
        r#"{{:harp/format "0.0.0-alpha"
 :package {{:identity "hara:example/jvm-only"
       :version "1.0.0"
       :provenance {{:repository "https://github.com/example/jvm-only"
                    :commit "0123456789abcdef0123456789abcdef01234567"}}}}
 :files {{"artifacts/provider.jar" {{:sha256 "{JVM_SHA}" :size 4}}}}
 :variants
 {{:jvm
   {{:variant/artifact
 {{:artifact/type :jar
  :artifact/path "artifacts/provider.jar"
  :artifact/sha256 "{JVM_SHA}"
  :artifact/target "java-21"
  :artifact/abi "hara.provider.jvm.v1"
  :artifact/entry-point "example.provider.HaraProvider"}}
:variant/required-capabilities #{{}}}}}}}}"#
    ))
    .unwrap();
    let error = jvm_only
        .select_variant(
            PackageRuntime::Wasm,
            &requirements("wasm32-wasi-preview1", "hta.v1", &[], &[]),
        )
        .unwrap_err();
    assert_eq!(error.code, "package/missing-variant");
}

#[test]
fn preflight_rejects_target_abi_capability_and_host_call_mismatches() {
    let manifest = PackageManifest::parse(&dual_runtime_manifest()).unwrap();
    let target = manifest
        .select_variant(
            PackageRuntime::Jvm,
            &requirements("java-17", "hara.provider.jvm.v1", &["db/connect"], &[]),
        )
        .unwrap_err();
    assert_eq!(target.code, "package/target-mismatch");

    let abi = manifest
        .select_variant(
            PackageRuntime::Jvm,
            &requirements("java-21", "hara.provider.jvm.v2", &["db/connect"], &[]),
        )
        .unwrap_err();
    assert_eq!(abi.code, "package/abi-mismatch");

    let capability = manifest
        .select_variant(
            PackageRuntime::Jvm,
            &requirements("java-21", "hara.provider.jvm.v1", &[], &[]),
        )
        .unwrap_err();
    assert_eq!(capability.code, "package/capability-denied");

    let host_call = manifest
        .select_variant(
            PackageRuntime::Wasm,
            &requirements("wasm32-wasi-preview1", "hta.v1", &["db/connect"], &[]),
        )
        .unwrap_err();
    assert_eq!(host_call.code, "package/host-call-denied");
}

#[test]
fn rejects_duplicate_variants_digest_drift_and_descriptor_authority() {
    let duplicate = PackageManifest::parse(&format!(
        r#"{{:harp/format "0.0.0-alpha"
 :package {{:identity "hara:example/duplicate"
       :version "1.0.0"
       :provenance {{:repository "https://github.com/example/duplicate"
                    :commit "0123456789abcdef0123456789abcdef01234567"}}}}
 :files {{"artifacts/provider.jar" {{:sha256 "{JVM_SHA}" :size 4}}}}
 :variants
 {{:jvm {{:variant/artifact
      {{:artifact/type :jar
       :artifact/path "artifacts/provider.jar"
       :artifact/sha256 "{JVM_SHA}"
       :artifact/target "java-21"
       :artifact/abi "hara.provider.jvm.v1"
       :artifact/entry-point "First"}}
     :variant/required-capabilities #{{}}}}
  :jvm {{:variant/artifact
      {{:artifact/type :jar
       :artifact/path "artifacts/provider.jar"
       :artifact/sha256 "{JVM_SHA}"
       :artifact/target "java-21"
       :artifact/abi "hara.provider.jvm.v1"
       :artifact/entry-point "Second"}}
     :variant/required-capabilities #{{}}}}}}}}"#
    ))
    .unwrap_err();
    assert_eq!(duplicate.code, "package/invalid-manifest");
    assert!(duplicate.detail.contains("Duplicate key"));

    let drift = dual_runtime_manifest().replacen(JVM_SHA, WASM_SHA, 1);
    let drift = PackageManifest::parse(&drift).unwrap_err();
    assert_eq!(drift.code, "package/digest-mismatch");

    let authority = dual_runtime_manifest().replace(
        ":descriptor {:operations [:provider/open :provider/close]}",
        ":descriptor {:socket \"ambient\"}",
    );
    let authority = PackageManifest::parse(&authority).unwrap_err();
    assert_eq!(authority.code, "package/invalid-descriptor");
}

#[test]
fn verifies_file_bytes_and_canonicalization_is_idempotent() {
    let manifest = PackageManifest::parse(&dual_runtime_manifest()).unwrap();
    let root =
        std::env::temp_dir().join(format!("hara-package-manifest-test-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(root.join("artifacts")).unwrap();
    fs::write(root.join("artifacts/provider.jar"), b"jvm!").unwrap();
    fs::write(root.join("artifacts/provider.hta"), b"wasm").unwrap();
    manifest.verify_files_at(&root).unwrap();
    fs::write(root.join("artifacts/provider.jar"), b"tampered").unwrap();
    let error = manifest.verify_files_at(&root).unwrap_err();
    assert!(matches!(
        error.code,
        "package/size-mismatch" | "package/digest-mismatch"
    ));
    fs::remove_dir_all(&root).unwrap();

    let canonical = manifest.canonical_edn().to_owned();
    let reparsed = PackageManifest::parse(&canonical).unwrap();
    assert_eq!(reparsed.canonical_edn(), canonical);
}
