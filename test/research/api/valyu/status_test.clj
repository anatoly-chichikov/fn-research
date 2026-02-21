(ns research.api.valyu.status-test
  (:require [clojure.test :refer [deftest is]]
            [jsonista.core :as json]
            [research.api.http :as request]
            [research.api.progress :as progress]
            [research.api.valyu.status :as status]
            [research.test.ids :as gen]))

(deftest ^{:doc "Ensure status retries transient errors"}
  the-status-retries-transient-errors
  (let [rng (gen/ids 18401)
        id (gen/cyrillic rng 6)
        key (gen/greek rng 5)
        base (gen/latin rng 6)
        fault (+ 500 (.nextInt rng 50))
        success (+ 200 (.nextInt rng 50))
        state (gen/armenian rng 6)
        body (json/write-value-as-string {:status state})
        count (atom 0)
        net (reify request/Requested
              (get [_ _ _]
                (swap! count inc)
                (delay (if (= @count 1)
                         {:status fault
                          :body (gen/greek rng 3)}
                         {:status success
                          :body body})))
              (post [_ _ _]
                (delay {})))
        item (status/make base key {:net net
                                    :log (progress/make)})]
    (with-redefs-fn {#'status/pause (fn [_ _] nil)
                     #'clojure.core/println (fn [& _] nil)}
      (fn []
        (let [data (status/status item id)]
          (is (= state (:status data))
              "status did not recover from transient error"))))))

(deftest ^{:doc "Ensure status retries missing status"}
  the-status-retries-missing-status
  (let [rng (gen/ids 18403)
        id (gen/cyrillic rng 6)
        key (gen/hebrew rng 5)
        base (gen/latin rng 6)
        success (+ 200 (.nextInt rng 50))
        state (gen/hiragana rng 6)
        body (json/write-value-as-string {:status state})
        count (atom 0)
        net (reify request/Requested
              (get [_ _ _]
                (swap! count inc)
                (if (= @count 1)
                  nil
                  (delay {:status success
                          :body body})))
              (post [_ _ _]
                (delay {})))
        item (status/make base key {:net net
                                    :log (progress/make)})]
    (with-redefs-fn {#'status/pause (fn [_ _] nil)
                     #'clojure.core/println (fn [& _] nil)}
      (fn []
        (let [data (status/status item id)]
          (is (= state (:status data))
              "status did not recover from missing status"))))))

(deftest ^{:doc "Ensure status throws on nil body instead of NPE"}
  the-status-throws-on-nil-body
  (let [rng (gen/ids 18405)
        id (gen/armenian rng 6)
        key (gen/greek rng 5)
        base (gen/latin rng 6)
        success (+ 200 (.nextInt rng 50))
        net (reify request/Requested
              (get [_ _ _]
                (delay {:status success
                        :body nil}))
              (post [_ _ _]
                (delay {})))
        item (status/make base key {:net net
                                    :log (progress/make)})]
    (with-redefs-fn {#'status/pause (fn [_ _] nil)
                     #'clojure.core/println (fn [& _] nil)}
      (fn []
        (is (thrown? clojure.lang.ExceptionInfo
                     (status/status item id))
            "status did not throw on nil body")))))
