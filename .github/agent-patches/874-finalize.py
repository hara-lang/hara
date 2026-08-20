#!/usr/bin/env python3
'Finalize #554 by promoting reviewed string routes into code.migrate rules.'

from __future__ import annotations

import hashlib
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BRANCH_FILES = {
    "external": ROOT / "core/lib/src/code/migrate/external.hal",
    "rule": ROOT / "core/lib/src/code/migrate/rule.hal",
    "external_test": ROOT / "core/lib/test/code/migrate_external_test.hal",
    "rules_test": ROOT / "core/lib/test/code/migrate_rules_test.hal",
}
CORPUS = ROOT / "core/spec/code-migrate/foundation-baa75a.edn"


def git_blob(path: Path) -> str:
    data = path.read_bytes()
    return hashlib.sha1(f"blob {len(data)}\0".encode("ascii") + data).hexdigest()


def replace_once(source: str, old: str, new: str, label: str) -> str:
    count = source.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one match, found {count}")
    return source.replace(old, new, 1)


def representative_string_route() -> tuple[str, str]:
    source = CORPUS.read_text(encoding="utf-8")
    match = re.search(
        r'\{([^{}]*:external/namespace "clojure\.string"[^{}]*)\}',
        source,
    )
    if not match:
        raise RuntimeError("cannot find a clojure.string route in foundation-baa75a")
    body = match.group(1)
    path = re.search(r':source/path "([^"]+)"', body)
    blob = re.search(r':source/blob "([0-9a-f]{40})"', body)
    if not path or not blob:
        raise RuntimeError("clojure.string route lacks exact source evidence")
    return path.group(1), blob.group(1)


def patch_external(path: Path) -> None:
    source = path.read_text(encoding="utf-8")
    if "(def +rules+" in source:
        return
    foundation_path, foundation_blob = representative_string_route()
    hara_path = ROOT / "core/lib/src/std/foundation/string.hal"
    hara_test_path = ROOT / "core/lib/test/std/foundation/string_test.hal"
    addition = f'''
(def +string-rule-evidence+
  [{{:foundation/repository "zcaudate-xyz/foundation-base"
     :foundation/revision "baa75aabd6a879753d7d5cb07271b1448271e7cb"
     :foundation/path "{foundation_path}"
     :foundation/blob "{foundation_blob}"
     :hara/path "core/lib/src/std/foundation/string.hal"
     :hara/blob "{git_blob(hara_path)}"
     :hara/test-path "core/lib/test/std/foundation/string_test.hal"
     :hara/test-blob "{git_blob(hara_test_path)}"
     :note "The complete external-route review admits only the documented portable string surface."}}])

(defn string-symbol-rule
  [entry]
  (let [source (first entry)
        target (second entry)]
    {{:id (keyword (str "external/clojure-string-" source))
     :kind :token
     :match {{:text (str "clojure.string/" source)}}
     :rewrite {{:op :replace-token
               :text (str "std.foundation.string/" target)}}
     :safety :review
     :priority 81
     :message
     (if (= source target)
       (str "Use the reviewed portable string symbol " target ".")
       (str "Rename the historical string symbol " source " to " target "."))
     :evidence +string-rule-evidence+}}))

(def +rules+
  (vec
   (concat
    [{{:id :external/clojure-string-dependency
       :kind :dependency-entry
       :match {{:text "clojure.string"}}
       :rewrite {{:op :replace-token
                 :text "std.foundation.string"}}
       :safety :review
       :priority 80
       :message "Use the reviewed portable string namespace."
       :evidence +string-rule-evidence+}}]
    (map string-symbol-rule
         (sort-by first (vec +string-symbols+))))))
'''
    source = replace_once(
        source,
        "\n(defn route-kind\n",
        addition + "\n(defn route-kind\n",
        "external rule insertion",
    )
    path.write_text(source, encoding="utf-8")


def patch_rule(path: Path) -> None:
    source = path.read_text(encoding="utf-8")
    if "[code.migrate.external :as external]" not in source:
        source = replace_once(
            source,
            "            [code.framework.navigation :as nav]\n"
            "            [code.migrate.rules :as declarations]))",
            "            [code.framework.navigation :as nav]\n"
            "            [code.migrate.external :as external]\n"
            "            [code.migrate.rules :as declarations]))",
            "rule require",
        )
    source = replace_once(
        source,
        "(def +ruleset+\n  (compile-rules declarations/+rules+))",
        "(def +ruleset+\n"
        "  (compile-rules\n"
        "   (vec (concat external/+rules+\n"
        "                declarations/+rules+))))",
        "combined ruleset",
    )
    path.write_text(source, encoding="utf-8")


def patch_external_test(path: Path) -> None:
    source = path.read_text(encoding="utf-8")
    marker = '\n(fact "route review preserves exact source evidence"\n'
    if "reviewed string routes compile to deterministic rules" not in source:
        addition = r'''
(fact "reviewed string routes compile to deterministic rules"
  [(count external/+rules+)
   (:id (first external/+rules+))
   (every? (fn [rule]
             (and (= :review (:safety rule))
                  (not (empty? (:evidence rule)))))
           external/+rules+)]
  => [19 :external/clojure-string-dependency true])

'''
        source = replace_once(
            source,
            marker,
            "\n" + addition + '(fact "route review preserves exact source evidence"\n',
            "external rule test",
        )
    path.write_text(source, encoding="utf-8")


def patch_rules_test(path: Path) -> None:
    source = path.read_text(encoding="utf-8")
    summary = re.compile(
        r'\(test-check "compiled rules retain safety and evidence totals".*?'
        r'\n\n   \(test-check "automatic matcher conflicts are rejected"',
        re.S,
    )
    replacement = r'''(test-check "compiled rules retain safety and evidence totals"
               [(translate/rules-summary)
                (every? (fn [entry] (not (empty? (:evidence entry))))
                        translate/rules)
                (= (vec (map :compiled/index translate/rules))
                   (vec (range (count translate/rules))))]
               [{:total 136 :safe 96 :review 27 :manual 13}
                true
                true])

   (test-check "automatic matcher conflicts are rejected"'''
    source, count = summary.subn(replacement, source, count=1)
    if count != 1:
        raise RuntimeError(f"rules summary test: expected one match, found {count}")

    if "reviewed clojure.string routes rewrite aliases" not in source:
        marker = '   (test-check "supported translations are idempotent"\n'
        addition = r'''   (test-check "reviewed clojure.string routes rewrite aliases and historical names"
               (let [input
                     "(ns demo (:require [clojure.string :as str]))\n[str/blank? str/triml str/trimr str/trim-newline clojure.string/split-lines]"
                     first-pass
                     (translate/translate-source input {:mode :review})
                     second-pass
                     (translate/translate-source
                      (:output first-pass)
                      {:mode :review})]
                 [(:output first-pass)
                  (set (map :rule/id (:rules/applied first-pass)))
                  (:changed second-pass)
                  (count (:rules/applied second-pass))])
               ["(ns demo (:require [std.foundation.string :as str]))\n[std.foundation.string/blank? std.foundation.string/trim-left std.foundation.string/trim-right std.foundation.string/trim-newlines std.foundation.string/split-lines]"
                #{:external/clojure-string-dependency
                  :external/clojure-string-blank?
                  :external/clojure-string-split-lines
                  :external/clojure-string-trim-newline
                  :external/clojure-string-triml
                  :external/clojure-string-trimr}
                false
                0])

'''
        source = replace_once(
            source,
            marker,
            addition + marker,
            "reviewed string translation test",
        )
    path.write_text(source, encoding="utf-8")


def main() -> int:
    patch_external(BRANCH_FILES["external"])
    patch_rule(BRANCH_FILES["rule"])
    patch_external_test(BRANCH_FILES["external_test"])
    patch_rules_test(BRANCH_FILES["rules_test"])
    Path(__file__).unlink()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
