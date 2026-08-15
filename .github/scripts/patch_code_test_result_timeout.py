from __future__ import annotations

import argparse
from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one anchor, found {count}")
    return text.replace(old, new, 1)


PROCESS_SOURCE = r'''(ns code.test.base.process
  (:require [code.test.checker.common :as common]
            [std.foundation :as foundation]))

(defn evaluate
  [input]
  (try
    {:type :code/test
     :status :success
     :data (if (map? input)
             ((or (:function input) (fn [] (:form input))))
             (input))}
    (catch Throwable error
      {:type :code/test :status :exception :data error})))

(defn synchronization-options
  [options]
  (let [context (merge {:test {:phase :settle}}
                       (or (:context options) {}))]
    (if (has? options :timeout)
      {:timeout (:timeout options)
       :context context}
      {:context context})))

(defn settle-result
  "Normalizes one completed assertion value through the native Result timeout path."
  ([value]
   (settle-result value {}))
  ([value options]
   (foundation/res-synchronize value (synchronization-options options))))

(defn settle
  "Returns the settled value, throwing the native Error for unsuccessful outcomes."
  ([value]
   (settle value {}))
  ([value options]
   (deref (settle-result value options))))

(defn timeout-error?
  [error]
  (= :result/timeout (get (ex-data error) :code)))

(defn timeout-milliseconds
  [outcome options]
  (or (get (ex-data (foundation/res-error-value outcome)) :timeout)
      (get (foundation/res-context outcome) :result/timeout)
      (:timeout options)))

(defn timeout-check-result
  [outcome reported-expected options]
  {:pass false
   :status :timeout
   :actual nil
   :expected reported-expected
   :timeout (timeout-milliseconds outcome options)})

(defn verification-result
  [checker reported-expected outcome]
  (if (foundation/res-error? outcome)
    (let [error (foundation/res-error-value outcome)
          evaluation {:type :code/test
                      :status :exception
                      :data error}
          verification (common/verify checker evaluation)
          passing (common/succeeded? verification)]
      {:pass passing
       :status (if passing :success :error)
       :actual nil
       :expected reported-expected
       :error (if passing nil (ex-message error))
       :error-data (if passing nil (ex-data error))})
    (let [actual (foundation/res-data outcome)
          evaluation {:type :code/test
                      :status :success
                      :data actual}
          verification (common/verify checker evaluation)]
      {:pass (common/succeeded? verification)
       :status (:status verification)
       :actual actual
       :expected reported-expected})))

(defn check
  "Evaluates a thunk and verifies its canonical code.test result envelope."
  ([thunk expected]
   (check thunk expected {}))
  ([thunk expected options]
   (let [checker (common/->checker expected)
         reported-expected (if (common/checker? expected)
                             (str expected)
                             expected)
         outcome
         (try
           (settle-result (thunk) options)
           (catch Throwable error
             (foundation/res-error
              error
              {:test {:phase :evaluation}})))]
     (if (foundation/res-timeout? outcome)
       (timeout-check-result outcome reported-expected options)
       (verification-result checker reported-expected outcome)))))

(defn checks-pass?
  [checks]
  (reduce (fn [passing check-result]
            (and passing (:pass check-result)))
          true
          checks))

(defn checks-error?
  [checks]
  (reduce (fn [errored check-result]
            (or errored (= :error (:status check-result))))
          false
          checks))

(defn checks-timeout?
  [checks]
  (reduce (fn [timed-out check-result]
            (or timed-out (= :timeout (:status check-result))))
          false
          checks))

(defn normalize-checks
  [value]
  (if (vector? value)
    value
    (if (map? value)
      [value]
      [{:pass true :actual value :expected :returned}])))

(defn clock-now
  [options]
  (let [now (:work/now options)]
    (if now (now) nil)))

(defn run-hook
  [hook options]
  (if hook
    (settle (hook) options)
    nil))

(defn run-hooks
  [hooks options]
  (reduce (fn [output hook]
            (do (run-hook hook options) output))
          nil
          hooks))

(defn fact-cancelled?
  [fact options]
  (if (:cancelled options) true false))

(defn fact-result-identity
  [fact]
  {:namespace (:namespace fact)
   :name (:name fact)
   :meta (:meta fact)})

(defn error-result
  [fact error]
  (let [timed-out (timeout-error? error)]
    (merge (fact-result-identity fact)
           {:status (if timed-out :timeout :error)
            :checks []
            :error (str error)}
           (if timed-out
             {:timeout (get (ex-data error) :timeout)}
             {}))))

(defn execute-fact-body
  [fact options]
  (try
    (let [checks
          (normalize-checks
           (settle ((:function fact) options) options))
          status (if (checks-timeout? checks)
                   :timeout
                   (if (checks-error? checks)
                     :error
                     (if (checks-pass? checks)
                       :passed
                       :failed)))]
      (merge (fact-result-identity fact)
             {:status status
              :checks checks}))
    (catch Throwable error
      (error-result fact error))))

(defn timed-result
  [result start options]
  (let [end (clock-now options)]
    (if (and start end)
      (assoc result :elapsed (- end start))
      result)))

(defn run-fact
  "Runs one fact and returns a structured test-domain result."
  ([fact]
   (run-fact fact {}))
  ([fact options]
   (let [identity (fact-result-identity fact)
         start (clock-now options)
         result
         (if (:skip (:meta fact))
           (merge identity {:status :skipped :checks []})
           (if (fact-cancelled? fact options)
             (merge identity {:status :cancelled :checks []})
             (try
               (let [checks
                     (try
                       (do
                         (run-hooks [(:before-each options)
                                     (:before (:meta fact))]
                                    options)
                         (normalize-checks
                          (settle ((:function fact) options) options)))
                       (finally
                         (run-hooks [(:after (:meta fact))
                                     (:after-each options)]
                                    options)))
                     status (if (checks-timeout? checks)
                              :timeout
                              (if (checks-error? checks)
                                :error
                                (if (checks-pass? checks)
                                  :passed
                                  :failed)))]
                 (merge identity {:status status :checks checks}))
               (catch Throwable error
                 (error-result fact error)))))
         end (clock-now options)
         timed-result (if (and start end)
                        (assoc result :elapsed (- end start))
                        result)]
     timed-result)))

(defn infer-function [form] (fn [] form))
(defn attach-meta [result metadata] (assoc result :meta metadata))
(defn evaluate-on-error [result] result)
(defn process [input] (evaluate input))
(defn collect [results] (vec results))
(defn skip-check [check] (assoc check :status :skipped))
(def run-check check)
'''


def test_candidate() -> str:
    path = Path("core/lib/test/code/test_test.hal")
    text = path.read_text()
    text = replace_once(
        text,
        '''  (:require
            [std.work :as work]))
''',
        '''  (:require [code.test.base.process :as process]
            [std.work :as work]))
''',
        "code.test timeout process require",
    )
    old = '''(def timeout-result
  (check
   (fn [] (promise/from 42))
   42
   {:work/timeout-promise
    (fn [promise milliseconds]
      {:promise (promise/from {:test/status :timeout})
       :timeout milliseconds})
    :work/cancel-timeout (fn [timeout] timeout)
    :timeout 25}))
'''
    new = '''(def timeout-outcome
  (process/settle-result
   (promise/delay 100 (fn [] 42))
   {:timeout 0}))

(def timeout-result
  (check
   (fn [] (promise/delay 100 (fn [] 42)))
   42
   {:timeout 0}))
'''
    text = replace_once(text, old, new, "code.test timeout setup")
    old = '''   (test-check "generic work timeout capabilities classify diagnostics"
                 [(get timeout-result :status)
                  (get timeout-result :pass)
                  (get timeout-result :timeout)]
                 [:timeout false 25])
'''
    new = '''   (test-check "native Result timeout classification replaces TimeoutValue"
                 [(std.foundation/res-timeout? timeout-outcome)
                  (get (ex-data
                        (std.foundation/res-error-value timeout-outcome))
                       :code)
                  (get timeout-result :status)
                  (get timeout-result :pass)
                  (get timeout-result :timeout)]
                 [true :result/timeout :timeout false 0])
'''
    text = replace_once(text, old, new, "code.test timeout assertions")
    return text


def write_candidates() -> None:
    target = Path("core/target/code-test-result-timeout")
    target.mkdir(parents=True, exist_ok=True)
    (target / "process.hal").write_text(PROCESS_SOURCE)
    (target / "test_test.hal").write_text(test_candidate())
    print(target)


def apply() -> None:
    target = Path("core/target/code-test-result-timeout")
    process = (target / "process.hal").read_text() if target.exists() else PROCESS_SOURCE
    test = (target / "test_test.hal").read_text() if target.exists() else test_candidate()
    Path("core/lib/src/code/test/base/process.hal").write_text(process)
    Path("core/rust/hal-src/code/test/base/process.hal").write_text(process)
    Path("core/lib/test/code/test_test.hal").write_text(test)
    print("code.test Result timeout migration applied")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=("candidate", "apply"))
    args = parser.parse_args()
    if args.command == "candidate":
        write_candidates()
    else:
        apply()


if __name__ == "__main__":
    main()
