from __future__ import annotations

import argparse
import re
from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one anchor, found {count}")
    return text.replace(old, new, 1)


def regex_replace_once(text: str, pattern: str, replacement: str, label: str) -> str:
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.DOTALL)
    if count != 1:
        raise RuntimeError(f"{label}: expected one regex anchor, found {count}")
    return updated


COMMON_RESULT_DATA = r'''(defn result-data [result]
  (cond
    (res-success? result) (res-data result)
    (res-error? result) (res-error-value result)
    (and (map? result)
         (= :code/test (:type result))
         (has? result :data))
    (:data result)
    :else result))
'''


COMMON_RESULT_HELPERS = r'''
(declare checker-descriptor)

(defn native-error?
  [value]
  (= :hara/Error (type value)))

(defn descriptor-value
  [value]
  (cond
    (checker? value) (checker-descriptor value)
    (fn? value) (str value)
    (regexp? value) {:descriptor/type :regexp
                     :descriptor/pattern (Regex/pattern value)}
    (map? value)
    (reduce-kv (fn [output key nested]
                 (assoc output key (descriptor-value nested)))
               {}
               value)
    (vector? value) (mapv descriptor-value value)
    (set? value) (into #{} (map descriptor-value value))
    (list? value) (vec (map descriptor-value value))
    (native-error? value)
    {:descriptor/type :error
     :descriptor/error-type (type value)
     :descriptor/message (ex-message value)
     :descriptor/data (or (ex-data value) {})}
    :else value))

(defn checker-descriptor
  "Returns portable data describing a local executable Checker."
  [checker]
  {:checker/type (:tag checker)
   :checker/expected (descriptor-value (:expect checker))
   :checker/form (descriptor-value (:form checker))})

(defn error-checker?
  [checker]
  (has? #{:throws :raises} (:tag checker)))

(defn comparison-context
  [checker actual]
  {:test {:checker (checker-descriptor checker)
          :actual actual}
   :failures []})

(defn invoke-checker-result
  [checker actual]
  (let [context (comparison-context checker actual)]
    (try
      (res-success (boolean (checker actual)) context)
      (catch Throwable error
        (res-error error context)))))

(defn compare-result
  "Compares one completed evaluation Result and returns Result<boolean>."
  [checker evaluation]
  (if (res-error? evaluation)
    (if (error-checker? checker)
      (invoke-checker-result checker (res-error-value evaluation))
      (res-with-context evaluation
                        {:test {:checker (checker-descriptor checker)}
                         :failures []}))
    (invoke-checker-result checker (res-data evaluation))))

'''


COMMON_SUCCEEDED = r'''(defn succeeded? [result]
  (if (res? result)
    (and (res-success? result) (= true (res-data result)))
    (and (= :success (:status result)) (= true (:data result)))))
'''


COMMON_THROWS = r'''(defn throws
  ([] (throws :hara/Error nil))
  ([error-type] (throws error-type nil))
  ([error-type message]
   (checker {:tag :throws
             :doc "Checks a captured native Error type and optional message."
             :fn (fn [result]
                   (let [error
                         (cond
                           (native-error? result) result
                           (res-error? result) (res-error-value result)
                           (and (map? result)
                                (= :exception (:status result)))
                           (:data result)
                           :else nil)]
                     (and error
                          (or (= error-type :hara/Error)
                              (= error-type (type error)))
                          (if message
                            (= message (ex-message error))
                            true))))
             :expect {:exception error-type :message message}})))
'''


COLLECTION_THROWS_INFO = r'''(defn throws-info
  ([] (throws-info {}))
  ([expected]
   (common/checker
    {:tag :raises
     :doc "Checks the data carried by a captured native Error."
     :fn (fn [result]
           (let [error
                 (cond
                   (common/native-error? result) result
                   (res-error? result) (res-error-value result)
                   (and (map? result)
                        (= :exception (:status result)))
                   (:data result)
                   :else nil)]
             (and error
                  (map? (ex-data error))
                  ((contains expected) (ex-data error)))))
     :expect {:exception :hara/Error :data expected}})))
'''


PROCESS_SETTLEMENT = r'''(defn settlement-context
  [options]
  (merge {:test {:phase :evaluation}}
         (or (:result/context options) {})))

(defn synchronization-options
  [options]
  (let [context (settlement-context options)]
    (if (has? options :timeout)
      {:timeout (:timeout options)
       :context context}
      {:context context})))

(defn settle-result
  "Normalizes a completed value or Promise through native Result synchronization."
  ([value]
   (settle-result value {}))
  ([value options]
   (res-synchronize value (synchronization-options options))))

(defn settle
  "Returns the successful settled value or throws the contained native Error."
  ([value]
   (settle value {}))
  ([value options]
   (deref (settle-result value options))))

(defn evaluation-result
  [thunk options]
  (try
    (settle-result (thunk) options)
    (catch Throwable error
      (res-error error (settlement-context options)))))

(defn reported-expected
  [expected]
  (if (common/checker? expected)
    (str expected)
    expected))

(defn check-operation
  [thunk expected options]
  (let [checker (common/->checker expected)
        evaluation (evaluation-result thunk options)
        comparison (common/compare-result checker evaluation)]
    {:checker checker
     :evaluation evaluation
     :comparison comparison
     :expected (reported-expected expected)}))

(defn check-result
  "Evaluates and compares once, returning one native Result<boolean>."
  ([thunk expected]
   (check-result thunk expected {}))
  ([thunk expected options]
   (:comparison (check-operation thunk expected options))))

(defn legacy-check-map
  "Temporary adapter for reporters that have not yet migrated to Result checks."
  [operation]
  (let [evaluation (:evaluation operation)
        comparison (:comparison operation)
        expected (:expected operation)
        actual (if (res-success? evaluation) (res-data evaluation) nil)]
    (cond
      (res-timeout? comparison)
      (let [error (res-error-value comparison)
            data (or (ex-data error) {})]
        {:pass false
         :status :timeout
         :actual actual
         :expected expected
         :timeout (get data :timeout)
         :error (ex-message error)
         :error-data data
         :comparison comparison})

      (res-error? comparison)
      (let [error (res-error-value comparison)]
        {:pass false
         :status :error
         :actual actual
         :expected expected
         :error (ex-message error)
         :error-data (ex-data error)
         :comparison comparison})

      :else
      {:pass (= true (res-data comparison))
       :status :success
       :actual actual
       :expected expected
       :comparison comparison})))

(defn check
  "Compatibility adapter over the canonical Result-native comparison path."
  ([thunk expected]
   (check thunk expected {}))
  ([thunk expected options]
   (legacy-check-map (check-operation thunk expected options))))
'''


PROCESS_AGGREGATION = r'''

;; Result-native aggregate interpretation. These definitions intentionally
;; replace the compatibility implementations above while fact/report migration
;; remains incremental.
(defn check-comparison
  [check]
  (if (res? check)
    check
    (if (and (map? check) (res? (:comparison check)))
      (:comparison check)
      nil)))

(defn comparison-status
  [check]
  (let [comparison (check-comparison check)]
    (cond
      (res-timeout? comparison) :timeout
      (res-error? comparison) :error
      (res-success? comparison)
      (if (= true (res-data comparison)) :passed :failed)
      (and (map? check) (= :timeout (:status check))) :timeout
      (and (map? check) (= :error (:status check))) :error
      (and (map? check) (:pass check)) :passed
      :else :failed)))

(defn checks-pass?
  [checks]
  (every? (fn [check]
            (= :passed (comparison-status check)))
          checks))

(defn checks-error?
  [checks]
  (any? (fn [check]
          (= :error (comparison-status check)))
        checks))

(defn checks-timeout?
  [checks]
  (any? (fn [check]
          (= :timeout (comparison-status check)))
        checks))

(defn normalize-checks
  [value]
  (cond
    (vector? value) value
    (res? value) [value]
    (map? value) [value]
    :else [{:pass true :actual value :expected :returned}]))
'''


CODE_TEST_EXPORT = "(def check-result process/check-result)\n"


RESULT_TEST = r'''(ns code.test.result-timeout-test
  (:use code.test)
  (:require [code.test.base.process :as process]))

(fact "returns Result<boolean> for passing and failing comparisons"
  (let [passing (process/check-result (fn [] 42) 42)
        failing (process/check-result (fn [] 42) 43)]
    [(res? passing)
     (res-success? passing)
     (res-data passing)
     (res-success? failing)
     (res-data failing)])
  => [true true true true false])

(fact "propagates evaluation errors except to error checkers"
  (let [ordinary
        (process/check-result
         (fn [] (throw (ex-info "boom" {:code :demo/boom})))
         42)
        expected-error
        (process/check-result
         (fn [] (throw (ex-info "boom" {:code :demo/boom})))
         (throws :hara/Error "boom"))]
    [(res-error? ordinary)
     (= :demo/boom
        (get (ex-data (res-error-value ordinary)) :code))
     (res-success? expected-error)
     (res-data expected-error)])
  => [true true true true])

(fact "derives timeout before generic error and mismatch statuses"
  (let [timeout (res-error
                 (ex-info "timed out"
                          {:code :result/timeout
                           :timeout 25}))
        errored (res-error
                 (ex-info "boom" {:code :demo/boom}))
        failed (res-success false)
        passed (res-success true)]
    [(process/comparison-status timeout)
     (process/comparison-status errored)
     (process/comparison-status failed)
     (process/comparison-status passed)
     (process/checks-timeout? [timeout])
     (process/checks-error? [errored])
     (process/checks-pass? [passed])])
  => [:timeout :error :failed :passed true true true])

(fact "keeps the old map adapter on the same comparison Result"
  (let [legacy (process/check (fn [] 42) 43)]
    [(:pass legacy)
     (:status legacy)
     (res? (:comparison legacy))
     (res-data (:comparison legacy))])
  => [false :success true false])

(pr-str (run '[code.test.result-timeout-test]))
'''


AUDIT = r'''#!/usr/bin/env bash
set -euo pipefail

failed=0

if git grep -n -F 'TimeoutValue' -- core/lib/src/code/test core/rust/hal-src/code/test; then
  echo 'code.test still contains the superseded TimeoutValue wrapper.' >&2
  failed=1
fi

if ! git grep -n -F '(res-synchronize value' -- core/lib/src/code/test/base/process.hal >/dev/null; then
  echo 'code.test settlement is not routed through res-synchronize.' >&2
  failed=1
fi

if ! git grep -n -F '(defn check-result' -- core/lib/src/code/test/base/process.hal >/dev/null; then
  echo 'Result-native code.test comparison entrypoint is missing.' >&2
  failed=1
fi

exit "$failed"
'''


def transform_common(text: str) -> str:
    if "(defn compare-result" in text:
        return text
    text = regex_replace_once(
        text,
        r"\(defn result-data \[result\]\n.*?\n\s*result\)\)\n",
        COMMON_RESULT_DATA,
        "common result-data",
    )
    text = text.replace(
        "(defn verify [checker result]\n",
        COMMON_RESULT_HELPERS + "(defn verify [checker result]\n",
        1,
    )
    text = regex_replace_once(
        text,
        r"\(defn succeeded\? \[result\]\n.*?\)\)\n",
        COMMON_SUCCEEDED,
        "common succeeded?",
    )
    text = regex_replace_once(
        text,
        r"\(defn throws\n.*?\n\s*:expect \{:exception error-type :message message\}\}\)\)\)\n",
        COMMON_THROWS,
        "common throws",
    )
    return text


def transform_collection(text: str) -> str:
    if "Checks the data carried by a captured native Error" in text:
        return text
    return regex_replace_once(
        text,
        r"\(defn throws-info\n.*?\n\s*:expect \{:exception :hara/Error :data expected\}\}\)\)\)\n?",
        COLLECTION_THROWS_INFO,
        "collection throws-info",
    )


def transform_process(text: str) -> str:
    if "(defn check-result" in text:
        return text
    text = text.replace("(defstruct TimeoutValue [milliseconds])\n\n", "", 1)
    text = regex_replace_once(
        text,
        r"\(defn timed-settle\n.*?\n\(defn check\n",
        PROCESS_SETTLEMENT + "\n(defn check\n",
        "process settlement",
    )
    text = regex_replace_once(
        text,
        r"\(defn check\n.*?\n\(defn checks-pass\?",
        PROCESS_SETTLEMENT.split("(defn check\n", 1)[1]
        if False
        else PROCESS_SETTLEMENT.rsplit("\n(defn check\n", 1)[0],
        "unused",
    )
    return text


def transform_process_precise(text: str) -> str:
    if "(defn check-result" in text:
        return text
    text = text.replace("(defstruct TimeoutValue [milliseconds])\n\n", "", 1)
    settlement_pattern = r"\(defn timed-settle\n.*?\n\(defn check\n"
    marker = "__CHECK_MARKER__"
    text = regex_replace_once(
        text,
        settlement_pattern,
        PROCESS_SETTLEMENT + "\n" + marker + "\n",
        "process settlement",
    )
    text = regex_replace_once(
        text,
        marker + r"\n.*?\n\(defn checks-pass\?",
        "(defn checks-pass?",
        "legacy check removal",
    )
    return text.rstrip() + "\n" + PROCESS_AGGREGATION.strip() + "\n"


def transform_facade(text: str) -> str:
    if "(def check-result process/check-result)" in text:
        return text
    return replace_once(
        text,
        "(def check process/check)\n",
        "(def check process/check)\n" + CODE_TEST_EXPORT,
        "code.test check-result export",
    )


def candidates() -> dict[str, str]:
    return {
        "common.hal": transform_common(
            Path("core/lib/src/code/test/checker/common.hal").read_text()
        ),
        "collection.hal": transform_collection(
            Path("core/lib/src/code/test/checker/collection.hal").read_text()
        ),
        "process.hal": transform_process_precise(
            Path("core/lib/src/code/test/base/process.hal").read_text()
        ),
        "test.hal": transform_facade(Path("core/lib/src/code/test.hal").read_text()),
    }


def write_candidates() -> None:
    root = Path("core/target/code-test-result-timeout")
    root.mkdir(parents=True, exist_ok=True)
    for name, content in candidates().items():
        (root / name).write_text(content)
    print(root)


def apply() -> None:
    generated = candidates()
    targets = {
        "common.hal": [
            Path("core/lib/src/code/test/checker/common.hal"),
            Path("core/rust/hal-src/code/test/checker/common.hal"),
        ],
        "collection.hal": [
            Path("core/lib/src/code/test/checker/collection.hal"),
            Path("core/rust/hal-src/code/test/checker/collection.hal"),
        ],
        "process.hal": [
            Path("core/lib/src/code/test/base/process.hal"),
            Path("core/rust/hal-src/code/test/base/process.hal"),
        ],
        "test.hal": [
            Path("core/lib/src/code/test.hal"),
            Path("core/rust/hal-src/code/test.hal"),
        ],
    }
    for name, paths in targets.items():
        for path in paths:
            path.write_text(generated[name])

    Path("core/lib/test/code/test/result_timeout_test.hal").write_text(RESULT_TEST)
    Path("scripts/audit-code-test-result-timeout.sh").write_text(AUDIT)
    print("Result-native code.test timeout integration applied")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=("candidate", "apply"))
    arguments = parser.parse_args()
    if arguments.command == "candidate":
        write_candidates()
    else:
        apply()


if __name__ == "__main__":
    main()
