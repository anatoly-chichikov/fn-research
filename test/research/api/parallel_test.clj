(ns research.api.parallel-test
  (:require [clojure.string :as str]
            [clojure.test :refer [deftest is]]
            [jsonista.core :as json]
            [org.httpkit.client :as http]
            [research.api.parallel :as parallel]
            [research.api.research :as research]
            [research.api.response :as response]
            [research.test.ids :as gen]))

(deftest the-parallel-returns-client
  (let [rng (gen/ids 16001)
        key (gen/cyrillic rng 6)
        client (with-redefs [parallel/env (fn [_] key)]
                 (parallel/parallel))]
    (is (instance?
         research.api.parallel.Parallel
         client)
        "Parallel client was not created")))

(deftest the-parallel-uses-environment
  (let [rng (gen/ids 16003)
        key (gen/cyrillic rng 6)
        client (with-redefs [parallel/env (fn [_] key)]
                 (parallel/parallel))]
    (is (= key (:key client)) "Parallel key did not match environment")))

(deftest the-parallel-raises-without-key
  (let [raised (atom false)]
    (try
      (with-redefs [parallel/env (fn [_] "")]
        (parallel/parallel))
      (catch Exception _ (reset! raised true)))
    (is @raised "Parallel did not raise when key missing")))

(deftest the-parallel-start-returns-run-id
  (let [rng (gen/ids 16007)
        run (str "trun_" (gen/uuid rng))]
    (with-redefs [parallel/env (fn [_] "key")
                  http/post (fn [_ _]
                              (delay {:status 200
                                      :body (json/write-value-as-string
                                             {:run_id run})}))]
      (let [client (parallel/parallel)
            result (research/start client "query" "pro")]
        (is (= run result) "start did not return expected run_id")))))

(deftest the-parallel-start-accepts-accepted-status
  (let [rng (gen/ids 16008)
        run (str "trun_" (gen/uuid rng))]
    (with-redefs [parallel/env (fn [_] "key")
                  http/post (fn [_ _]
                              (delay {:status 202
                                      :body (json/write-value-as-string
                                             {:run_id run})}))]
      (let [client (parallel/parallel)
            result (research/start client "query" "pro")]
        (is (= run result) "Accepted status was not handled")))))

(deftest the-parallel-start-passes-query
  (let [rng (gen/ids 16009)
        query (gen/cyrillic rng 6)
        holder (atom "")]
    (with-redefs [parallel/env (fn [_] "key")
                  http/post (fn [_ opts]
                              (reset! holder (:body opts))
                              (delay {:status 200
                                      :body (json/write-value-as-string
                                             {:run_id "trun_x"})}))]
      (let [client (parallel/parallel)]
        (research/start client query "pro")))
    (let [data (json/read-value
                @holder
                (json/object-mapper {:decode-key-fn keyword}))]
      (is (= query (:input data)) "Query was not passed to create"))))

(deftest the-parallel-start-passes-processor
  (let [rng (gen/ids 16011)
        processor (gen/hiragana rng 5)
        holder (atom "")]
    (with-redefs [parallel/env (fn [_] "key")
                  http/post (fn [_ opts]
                              (reset! holder (:body opts))
                              (delay {:status 200
                                      :body (json/write-value-as-string
                                             {:run_id "trun_x"})}))]
      (let [client (parallel/parallel)]
        (research/start client "query" processor)))
    (let [data (json/read-value
                @holder
                (json/object-mapper {:decode-key-fn keyword}))]
      (is (= processor (:processor data))
          "Processor was not passed to create"))))

(deftest the-parallel-finish-returns-completed-response
  (let [rng (gen/ids 16013)
        run (str "trun_" (gen/uuid rng))
        body {:run {:run_id run
                    :status "completed"}
              :output {:content "result"
                       :basis []}}]
    (with-redefs [parallel/env (fn [_] "key")
                  http/get (fn [_ _]
                             (delay {:status 200
                                     :body (json/write-value-as-string
                                            body)}))]
      (let [client (parallel/parallel)
            item (research/finish client run)]
        (is (response/completed item)
            "Response was not marked as completed")))))

(deftest the-parallel-finish-returns-markdown
  (let [rng (gen/ids 16015)
        run (str "trun_" (gen/uuid rng))
        output (str "# " (gen/cyrillic rng 6))
        body {:run {:run_id run
                    :status "completed"}
              :output {:content output
                       :basis []}}]
    (with-redefs [parallel/env (fn [_] "key")
                  http/get (fn [_ _]
                             (delay {:status 200
                                     :body (json/write-value-as-string
                                            body)}))]
      (let [client (parallel/parallel)
            item (research/finish client run)]
        (is (= output (response/text item))
            "Markdown did not match API output")))))

(deftest the-parallel-stream-handles-empty-events
  (let [rng (gen/ids 16017)
        bytes (.getBytes "" "UTF-8")
        body (java.io.ByteArrayInputStream. bytes)
        client (with-redefs [parallel/env (fn [_] "key")
                             http/get (fn [_ _] {:status 200
                                                 :body body})]
                 (parallel/parallel))
        result (research/stream client (str "trun_" (gen/uuid rng)))]
    (is result "Stream did not complete")))

(deftest the-parallel-start-sends-output-description
  (let [rng (gen/ids 16021)
        holder (atom "")]
    (with-redefs [parallel/env (fn [_] "key")
                  http/post (fn [_ opts]
                              (reset! holder (:body opts))
                              (delay {:status 200
                                      :body (json/write-value-as-string
                                             {:run_id "trun_x"})}))]
      (let [client (parallel/parallel)]
        (research/start client (gen/cyrillic rng 6) "ultra")))
    (let [data (json/read-value
                @holder
                (json/object-mapper {:decode-key-fn keyword}))
          desc (get-in data [:task_spec :output_schema :description])]
      (is (str/includes? (str desc) "details")
          "Output description did not include detail guidance"))))

(deftest the-parallel-clean-removes-periods
  (let [rng (gen/ids 16019)
        text (gen/cyrillic rng 6)
        value (str text "." text ".")
        clean (parallel/clean value)]
    (is (not (str/includes? clean "."))
        "Parallel clean did not remove periods")))
