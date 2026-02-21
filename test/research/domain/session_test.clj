(ns research.domain.session-test
  (:require [clojure.test :refer [deftest is]]
            [research.domain.pending :as pending]
            [research.domain.result :as result]
            [research.domain.session :as session]
            [research.domain.task :as task]
            [research.test.ids :as gen]))

(deftest the-session-generates-unique-id
  (let [rng (gen/ids 12001)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        topic (gen/cyrillic rng 6)
        item (session/session {:topic topic
                               :tasks []
                               :created time})]
    (is (= 36 (count (session/id item)))
        "Session identifier length is incorrect")))

(deftest the-session-returns-provided-topic
  (let [rng (gen/ids 12003)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        topic (gen/hiragana rng 7)
        item (session/session {:id (gen/uuid rng)
                               :topic topic
                               :tasks []
                               :created time})]
    (is (= topic (session/topic item))
        "Session topic did not match provided value")))

(deftest the-session-extend-adds-task
  (let [rng (gen/ids 12005)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        topic (gen/greek rng 6)
        item (session/session {:id (gen/uuid rng)
                               :topic topic
                               :tasks []
                               :created time})
        query (gen/cyrillic rng 6)
        status (gen/cyrillic rng 6)
        language (gen/cyrillic rng 5)
        service (gen/cyrillic rng 4)
        summary (gen/cyrillic rng 6)
        value (result/->ResearchReport summary [])
        task (task/task {:id (gen/uuid rng)
                         :query query
                         :status status
                         :language language
                         :service service
                         :result (result/data value)
                         :created time})
        output (session/extend item task)]
    (is (= 1 (count (session/tasks output)))
        "Extended session did not contain one task")))

(deftest the-session-extend-preserves-id
  (let [rng (gen/ids 12007)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        topic (gen/cyrillic rng 6)
        ident (gen/uuid rng)
        item (session/session {:id ident
                               :topic topic
                               :tasks []
                               :created time})
        query (gen/cyrillic rng 6)
        status (gen/cyrillic rng 6)
        language (gen/cyrillic rng 5)
        service (gen/cyrillic rng 4)
        summary (gen/cyrillic rng 6)
        value (result/->ResearchReport summary [])
        task (task/task {:id (gen/uuid rng)
                         :query query
                         :status status
                         :language language
                         :service service
                         :result (result/data value)
                         :created time})
        output (session/extend item task)]
    (is (= ident (session/id output))
        "Extended session ID did not match original")))

(deftest the-session-serializes-topic
  (let [rng (gen/ids 12009)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        topic (gen/hiragana rng 6)
        item (session/session {:id (gen/uuid rng)
                               :topic topic
                               :tasks []
                               :created time})
        data (session/data item)]
    (is (= topic (:topic data))
        "Serialized topic did not match original")))

(deftest the-session-deserializes-correctly
  (let [rng (gen/ids 12011)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        topic (gen/hiragana rng 6)
        data {:id (gen/uuid rng)
              :topic topic
              :tasks []
              :created time}
        item (session/session data)]
    (is (= topic (session/topic item))
        "Deserialized topic did not match")))

(deftest the-session-pending-returns-empty
  (let [rng (gen/ids 12013)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        topic (gen/cyrillic rng 6)
        item (session/session {:id (gen/uuid rng)
                               :topic topic
                               :tasks []
                               :created time})]
    (is (not (.isPresent (session/pending item)))
        "Pending run was present for new session")))

(deftest the-session-start-sets-pending
  (let [rng (gen/ids 12015)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        run (gen/cyrillic rng 6)
        query (gen/hiragana rng 6)
        processor (gen/greek rng 6)
        language (gen/cyrillic rng 6)
        provider (gen/cyrillic rng 6)
        item (session/session {:id (gen/uuid rng)
                               :topic (gen/cyrillic rng 5)
                               :tasks []
                               :created time})
        hold (pending/pending {:run_id run
                               :query query
                               :processor processor
                               :language language
                               :provider provider})
        output (session/start item hold)]
    (is (= run (pending/id (.get (session/pending output))))
        "Pending run identifier did not match")))

(deftest the-session-clear-removes-pending
  (let [rng (gen/ids 12017)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        run (gen/cyrillic rng 6)
        query (gen/hiragana rng 6)
        processor (gen/greek rng 6)
        language (gen/cyrillic rng 6)
        provider (gen/cyrillic rng 6)
        item (session/session {:id (gen/uuid rng)
                               :topic (gen/cyrillic rng 5)
                               :tasks []
                               :created time
                               :pending {:run_id run
                                         :query query
                                         :processor processor
                                         :language language
                                         :provider provider}})
        output (session/reset item)]
    (is (not (.isPresent (session/pending output)))
        "Pending run was not cleared")))

(deftest the-session-serializes-pending
  (let [rng (gen/ids 12019)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        run (gen/cyrillic rng 6)
        query (gen/hiragana rng 6)
        processor (gen/greek rng 6)
        language (gen/cyrillic rng 6)
        provider (gen/cyrillic rng 6)
        hold (pending/pending {:run_id run
                               :query query
                               :processor processor
                               :language language
                               :provider provider})
        item (session/start (session/session {:id (gen/uuid rng)
                                              :topic (gen/cyrillic rng 5)
                                              :tasks []
                                              :created time})
                            hold)
        data (session/data item)]
    (is (= run (get-in data [:pending :run_id]))
        "Serialized pending run_id did not match")))

(deftest the-session-deserializes-pending
  (let [rng (gen/ids 12021)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        run (gen/cyrillic rng 6)
        query (gen/hiragana rng 6)
        processor (gen/greek rng 6)
        language (gen/cyrillic rng 6)
        provider (gen/cyrillic rng 6)
        data {:id (gen/uuid rng)
              :topic (gen/cyrillic rng 5)
              :tasks []
              :created time
              :pending {:run_id run
                        :query query
                        :processor processor
                        :language language
                        :provider provider}}
        item (session/session data)]
    (is (= run (pending/id (.get (session/pending item))))
        "Deserialized pending run did not match")))

(deftest the-session-returns-provided-query
  (let [rng (gen/ids 12023)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        topic (gen/cyrillic rng 6)
        query (gen/greek rng 7)
        item (session/session {:id (gen/uuid rng)
                               :topic topic
                               :tasks []
                               :created time
                               :query query})]
    (is (= query (session/query item))
        "Session query did not match provided value")))

(deftest the-session-returns-provided-processor
  (let [rng (gen/ids 12025)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        topic (gen/cyrillic rng 6)
        processor (gen/armenian rng 5)
        item (session/session {:id (gen/uuid rng)
                               :topic topic
                               :tasks []
                               :created time
                               :processor processor})]
    (is (= processor (session/processor item))
        "Session processor did not match provided value")))

(deftest the-session-returns-provided-language
  (let [rng (gen/ids 12027)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        topic (gen/cyrillic rng 6)
        language (gen/hiragana rng 5)
        item (session/session {:id (gen/uuid rng)
                               :topic topic
                               :tasks []
                               :created time
                               :language language})]
    (is (= language (session/language item))
        "Session language did not match provided value")))

(deftest the-session-returns-provided-provider
  (let [rng (gen/ids 12029)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        topic (gen/cyrillic rng 6)
        provider (gen/greek rng 5)
        item (session/session {:id (gen/uuid rng)
                               :topic topic
                               :tasks []
                               :created time
                               :provider provider})]
    (is (= provider (session/provider item))
        "Session provider did not match provided value")))

(deftest the-session-reconfigure-updates-provider
  (let [rng (gen/ids 12031)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        topic (gen/cyrillic rng 6)
        provider (gen/greek rng 5)
        processor (gen/armenian rng 5)
        item (session/session {:id (gen/uuid rng)
                               :topic topic
                               :tasks []
                               :created time
                               :provider (gen/cyrillic rng 4)
                               :processor (gen/cyrillic rng 4)})
        updated (session/reconfigure item {:provider provider
                                           :processor processor})]
    (is (= provider (session/provider updated))
        "Reconfigured provider did not match")))

(deftest the-session-serializes-research-params
  (let [rng (gen/ids 12033)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        topic (gen/cyrillic rng 6)
        query (gen/greek rng 7)
        processor (gen/armenian rng 5)
        language (gen/hiragana rng 5)
        provider (gen/cyrillic rng 5)
        item (session/session {:id (gen/uuid rng)
                               :topic topic
                               :tasks []
                               :created time
                               :query query
                               :processor processor
                               :language language
                               :provider provider})
        data (session/data item)]
    (is (= query (:query data))
        "Serialized query did not match original")))
