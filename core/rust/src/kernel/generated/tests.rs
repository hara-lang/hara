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
fn config_override_omits_selected_foundation_vars() {
    let config = GeneratedNamespaceConfig::configure(
        &parse_forms("(:config {:override [compile pointer]})").unwrap(),
    )
    .unwrap();
    assert!(config.excluded_foundation().contains("compile"));
    assert!(config.excluded_foundation().contains("pointer"));
    assert!(GeneratedNamespaceConfig::configure(
        &parse_forms("(:config {:blank true :override [compile]})").unwrap()
    )
    .unwrap_err()
    .contains("cannot be combined"));
}

#[test]
fn config_expose_selects_an_exact_foundation_surface() {
    let config = GeneratedNamespaceConfig::configure(
        &parse_forms("(:config {:expose [map reduce]})").unwrap(),
    )
    .unwrap();
    let exposed = config.exposed_foundation().unwrap();
    assert!(exposed.contains("map"));
    assert!(exposed.contains("reduce"));
    assert!(GeneratedNamespaceConfig::configure(
        &parse_forms("(:config {:override [map] :expose [reduce]})").unwrap()
    )
    .unwrap_err()
    .contains("cannot be combined"));
    assert!(GeneratedNamespaceConfig::configure(
        &parse_forms("(:config {:blank true :expose []})").unwrap()
    )
    .unwrap_err()
    .contains("cannot be combined"));
}

#[test]
fn refer_clojure_is_not_a_hara_namespace_clause() {
    assert!(GeneratedNamespaceConfig::configure(
        &parse_forms("(:refer-clojure :exclude [compile])").unwrap()
    )
    .unwrap_err()
    .contains("Unsupported ns clause: :refer-clojure"));
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
    assert_eq!(
        config
            .rewrite(parse_forms("test/run").unwrap().remove(0))
            .to_string(),
        "test/run"
    );
}

#[test]
fn coroutine_aliases_rewrite_to_fiber_control_forms() {
    let config = GeneratedNamespaceConfig::defaults();
    assert_eq!(
        config
            .rewrite(parse_forms("co/yield").unwrap().remove(0))
            .to_string(),
        "std.foundation.coroutine/yield"
    );
    assert_eq!(
        config
            .rewrite(parse_forms("co/await").unwrap().remove(0))
            .to_string(),
        "std.foundation.coroutine/await"
    );
}

#[test]
fn only_portable_foundation_shorthands_are_automatic() {
    let config = GeneratedNamespaceConfig::defaults();
    let mut foundation_aliases: Vec<_> = config
        .aliases()
        .into_iter()
        .filter(|(_, namespace)| namespace.starts_with("std.foundation."))
        .collect();
    foundation_aliases.sort();
    assert_eq!(
        foundation_aliases,
        vec![
            ("bytes".into(), "std.foundation.bytes".into()),
            ("co".into(), "std.foundation.coroutine".into()),
            ("promise".into(), "std.foundation.promise".into()),
            ("pretty".into(), "std.foundation.pretty".into()),
            ("str".into(), "std.foundation.string".into()),
        ]
    );

    let rebound = GeneratedNamespaceConfig::configure_with(
        &parse_forms("(:require [demo.kernel :as kernel])").unwrap(),
        |target| target == "demo.kernel",
    )
    .unwrap();
    assert!(rebound
        .aliases()
        .contains(&("kernel".into(), "demo.kernel".into())));
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
