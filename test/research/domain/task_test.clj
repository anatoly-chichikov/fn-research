(ns research.domain.task-test
  (:require [clojure.string :as str]
            [clojure.test :refer [deftest is]]
            [research.domain.result :as result]
            [research.domain.task :as task]
            [research.test.ids :as gen]))

(deftest the-task-generates-unique-id
  (let [rng (gen/ids 11001)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        query (gen/cyrillic rng 6)
        status (gen/cyrillic rng 6)
        language (gen/cyrillic rng 5)
        service (gen/cyrillic rng 4)
        summary (gen/cyrillic rng 6)
        value (result/->ResearchReport summary [])
        item (task/task {:query query
                         :status status
                         :language language
                         :service service
                         :result (result/data value)
                         :created time})]
    (is (= 36 (count (task/id item)))
        "Task identifier length is incorrect")))

(deftest the-task-returns-provided-query
  (let [rng (gen/ids 11003)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        query (gen/hiragana rng 7)
        status (gen/cyrillic rng 6)
        language (gen/cyrillic rng 5)
        service (gen/cyrillic rng 4)
        summary (gen/cyrillic rng 6)
        value (result/->ResearchReport summary [])
        item (task/task {:id (gen/uuid rng)
                         :query query
                         :status status
                         :language language
                         :service service
                         :result (result/data value)
                         :created time})
        text (task/query item)
        ok (and (str/includes? text language)
                (str/ends-with? text query))]
    (is ok "Task query did not include language and query")))

(deftest the-task-returns-provided-status
  (let [rng (gen/ids 11005)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        status (gen/greek rng 6)
        query (gen/cyrillic rng 6)
        language (gen/cyrillic rng 5)
        service (gen/cyrillic rng 4)
        summary (gen/cyrillic rng 6)
        value (result/->ResearchReport summary [])
        item (task/task {:id (gen/uuid rng)
                         :query query
                         :status status
                         :language language
                         :service service
                         :result (result/data value)
                         :created time})]
    (is (= status (task/status item))
        "Task status did not match provided value")))

(deftest the-task-complete-returns-new-task
  (let [rng (gen/ids 11007)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        query (gen/cyrillic rng 6)
        status (gen/cyrillic rng 6)
        language (gen/cyrillic rng 5)
        service (gen/cyrillic rng 4)
        summary (gen/cyrillic rng 5)
        value (result/->ResearchReport summary [])
        item (task/task {:id (gen/uuid rng)
                         :query query
                         :status status
                         :language language
                         :service service
                         :result (result/data value)
                         :created time})
        output (task/finish item value)]
    (is (= "completed" (task/status output))
        "Completed task status was not completed")))

(deftest the-task-complete-preserves-id
  (let [rng (gen/ids 11009)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        query (gen/cyrillic rng 6)
        status (gen/cyrillic rng 6)
        language (gen/cyrillic rng 5)
        service (gen/cyrillic rng 4)
        summary (gen/cyrillic rng 6)
        value (result/->ResearchReport summary [])
        item (task/task {:id (gen/uuid rng)
                         :query query
                         :status status
                         :language language
                         :service service
                         :result (result/data value)
                         :created time})
        output (task/finish item value)]
    (is (= (task/id item) (task/id output))
        "Completed task ID did not match original")))

(deftest the-task-complete-adds-timestamp
  (let [rng (gen/ids 11011)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        query (gen/cyrillic rng 6)
        status (gen/cyrillic rng 6)
        language (gen/cyrillic rng 5)
        service (gen/cyrillic rng 4)
        summary (gen/cyrillic rng 6)
        value (result/->ResearchReport summary [])
        item (task/task {:id (gen/uuid rng)
                         :query query
                         :status status
                         :language language
                         :service service
                         :result (result/data value)
                         :created time})
        output (task/finish item value)]
    (is (.isPresent (task/completed output))
        "Completed task timestamp was missing")))

(deftest the-task-omits-query-serialization
  (let [rng (gen/ids 11013)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        query (str (gen/hiragana rng 6)
                   "\n\nResearch:\n1. "
                   (gen/greek rng 4))
        status (gen/cyrillic rng 6)
        processor (gen/greek rng 6)
        language (gen/cyrillic rng 5)
        service (gen/cyrillic rng 4)
        summary (gen/cyrillic rng 6)
        value (result/->ResearchReport summary [])
        item (task/task {:id (gen/uuid rng)
                         :query query
                         :status status
                         :language language
                         :service service
                         :processor processor
                         :result (result/data value)
                         :created time})
        data (task/data item)
        brief (:brief data)
        items (:items brief)
        node (first items)]
    (is (and (contains? data :brief)
             (contains? brief :topic)
             (contains? brief :items)
             (= processor (:processor data))
             (contains? node :text)
             (contains? node :items)
             (not (contains? brief :text))
             (not (contains? data :query))
             (not (contains? data :result)))
        (str
         "Serialized task did not include brief or still included "
         "query or result"))))

(deftest ^{:doc "Ensure task renders nested brief items."}
  the-task-renders-nested-brief-items
  (let [rng (gen/ids 11017)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        topic (gen/cyrillic rng 6)
        first (gen/greek rng 5)
        inner (gen/armenian rng 5)
        second (gen/hiragana rng 5)
        status (gen/greek rng 6)
        language (gen/cyrillic rng 5)
        service (gen/cyrillic rng 4)
        summary (gen/cyrillic rng 6)
        value (result/->ResearchReport summary [])
        brief {:topic topic
               :items [{:text first
                        :items [{:text inner
                                 :items []}]}
                       {:text second
                        :items []}]}
        item (task/task {:id (gen/uuid rng)
                         :brief brief
                         :status status
                         :language language
                         :service service
                         :result (result/data value)
                         :created time})
        text (task/query item)
        ok (and (str/includes? text language)
                (str/includes? text topic)
                (str/includes? text first)
                (str/includes? text inner)
                (str/includes? text second))]
    (is ok "Nested brief was not rendered")))

(deftest the-task-prefers-explicit-topic-in-brief
  (let [rng (gen/ids 11021)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        mark (gen/hiragana rng 6)
        query (str (gen/greek rng 6)
                   "\n\nResearch:\n1. "
                   (gen/armenian rng 4))
        item (task/task {:id (gen/uuid rng)
                         :query query
                         :status (gen/greek rng 6)
                         :language (gen/cyrillic rng 5)
                         :service (gen/cyrillic rng 4)
                         :created time
                         :topic mark})
        brief (task/brief item)
        topic (:topic brief)]
    (is (= mark topic)
        "Task brief did not use explicit topic")))

(deftest the-task-deserializes-correctly
  (let [rng (gen/ids 11015)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        query (gen/cyrillic rng 7)
        status (gen/greek rng 6)
        language (gen/cyrillic rng 5)
        service (gen/cyrillic rng 4)
        summary (gen/cyrillic rng 6)
        value (result/->ResearchReport summary [])
        data {:id (gen/uuid rng)
              :query query
              :status status
              :language language
              :service service
              :result (result/data value)
              :created time}
        item (task/task data)
        text (task/query item)
        ok (and (str/includes? text language)
                (str/ends-with? text query))]
    (is ok "Deserialized query did not include language and query")))

(deftest ^{:doc "Ensure nested query items are parsed into brief items."}
  the-task-parses-nested-query-items
  (let [rng (gen/ids 11019)
        topic (gen/cyrillic rng 6)
        head (gen/greek rng 5)
        inner (gen/armenian rng 5)
        tail (gen/hiragana rng 5)
        pad (apply str (repeat 4 " "))
        query (str topic
                   "\n\nResearch:\n1. "
                   head
                   "\n"
                   pad
                   "1. "
                   inner
                   "\n2. "
                   tail)
        item (task/task {:id (gen/uuid rng)
                         :query query
                         :status (gen/greek rng 6)
                         :language (gen/cyrillic rng 5)
                         :service (gen/cyrillic rng 4)
                         :created "2026-01-01T00:00:00"})
        brief (task/brief item)
        items (:items brief)
        node (first items)
        peer (second items)
        ok (and (= head (:text node))
                (= inner (:text (first (:items node))))
                (= tail (:text peer)))]
    (is ok "Nested query items were not parsed")))
