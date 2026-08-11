use super::GeneratedNamespaceConfig;
use crate::kernel::parse_forms;

#[test]
fn configures_defaults_exclusions_aliases_and_requires_without_sources() {
    let forms = parse_forms(
        "(:intrinsics {:exclude [bytes] :aliases {string text}}) \
             (:require [hara.lib.string :as s :refer [trim]])",
    )
    .unwrap();
    let config = GeneratedNamespaceConfig::configure(&forms).unwrap();
    let rewritten = config.rewrite(
        parse_forms("(trim (s/trim (text/upper \" x \")))")
            .unwrap()
            .remove(0),
    );
    let display = format!("{rewritten:?}");
    assert!(display.contains("str/trim"));
    assert!(display.contains("str/upper"));
    assert!(display.contains("bytes/count") == false);
    assert!(GeneratedNamespaceConfig::configure(
        &parse_forms("(:require [missing.lib :as x])").unwrap()
    )
    .unwrap_err()
    .contains("missing generated namespace"));
}

#[test]
fn parses_config_clause_with_builtins_blank_and_intrinsics() {
    let forms = parse_forms(
        "(:config {:blank true \
                       :builtins [+ - = count get] \
                       :intrinsics {:exclude [bytes]}})",
    )
    .unwrap();
    let config = GeneratedNamespaceConfig::configure(&forms).unwrap();
    assert!(config.blank());
    assert_eq!(config.builtins(), &["+", "-", "=", "count", "get"]);
    assert_eq!(
        config
            .rewrite(parse_forms("bytes").unwrap().remove(0))
            .to_string(),
        "bytes"
    );
}

#[test]
fn records_used_namespaces_for_runtime_referral() {
    let config = GeneratedNamespaceConfig::configure_with(
        &parse_forms("(:use code.test)").unwrap(),
        |target| target == "code.test",
    )
    .unwrap();
    assert_eq!(config.required_namespaces(), &["code.test"]);
    assert_eq!(config.used_namespaces(), &["code.test"]);
    assert!(
        GeneratedNamespaceConfig::configure(&parse_forms("(:use [code.test])").unwrap())
            .unwrap_err()
            .contains(":use expects unqualified namespace symbols")
    );
}

#[test]
fn records_lazy_alias_without_an_eager_dependency() {
    let config = GeneratedNamespaceConfig::configure_with(
        &parse_forms("(:require [code.test :as test :lazy true])").unwrap(),
        |target| target == "code.test",
    )
    .unwrap();
    assert!(config.required_namespaces().is_empty());
    assert_eq!(config.lazy_target("test"), Some("code.test"));
}

#[test]
fn foundation_require_exclusions_remove_implicit_refers() {
    let config = GeneratedNamespaceConfig::configure(
        &parse_forms("(:require [std.foundation :refer :all :exclude [eval-in-ns]])").unwrap(),
    )
    .unwrap();
    assert!(config.excluded_foundation().contains("eval-in-ns"));
}

#[test]
fn native_aliases_are_universal_and_cannot_be_rebound() {
    let config =
        GeneratedNamespaceConfig::configure(&parse_forms("(:config {:blank true})").unwrap())
            .unwrap();
    assert_eq!(
        config
            .rewrite(parse_forms("Iter/iter-map").unwrap().remove(0))
            .to_string(),
        "std.native.Iter/iter-map"
    );
    assert!(GeneratedNamespaceConfig::configure(
        &parse_forms("(:require [std.native.Maths :as Iter])").unwrap()
    )
    .unwrap_err()
    .contains("Namespace alias already refers to std.native.Iter"));
}
