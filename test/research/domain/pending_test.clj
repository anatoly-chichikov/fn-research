(ns research.domain.pending-test
  (:require [clojure.string :as str]
            [clojure.test :refer [deftest is]]
            [research.domain.pending :as pending]
            [research.test.ids :as gen]))

(deftest the-pending-returns-identifier
  (let [rng (gen/ids 13001)
        run (gen/cyrillic rng 6)
        query (str (gen/hiragana rng 6)
                   "\n\nResearch:\n1. "
                   (gen/greek rng 4))
        processor (gen/greek rng 6)
        language (gen/cyrillic rng 6)
        provider (gen/cyrillic rng 6)
        item (pending/pending {:run_id run
                               :query query
                               :processor processor
                               :language language
                               :provider provider})]
    (is (= run (pending/id item))
        "Pending identifier did not match provided value")))

(deftest the-pending-returns-query
  (let [rng (gen/ids 13003)
        run (gen/cyrillic rng 6)
        query (str (gen/hiragana rng 6)
                   "\n\nResearch:\n1. "
                   (gen/greek rng 4))
        processor (gen/greek rng 6)
        language (gen/cyrillic rng 6)
        provider (gen/cyrillic rng 6)
        item (pending/pending {:run_id run
                               :query query
                               :processor processor
                               :language language
                               :provider provider})
        text (pending/query item)
        ok (and (str/includes? text language)
                (str/ends-with? text query))]
    (is ok "Pending query did not include language and query")))

(deftest the-pending-returns-processor
  (let [rng (gen/ids 13005)
        run (gen/cyrillic rng 6)
        query (str (gen/hiragana rng 6)
                   "\n\nResearch:\n1. "
                   (gen/greek rng 4))
        processor (gen/greek rng 6)
        language (gen/cyrillic rng 6)
        provider (gen/cyrillic rng 6)
        item (pending/pending {:run_id run
                               :query query
                               :processor processor
                               :language language
                               :provider provider})]
    (is (= processor (pending/processor item))
        "Pending processor did not match provided value")))

(deftest the-pending-returns-language
  (let [rng (gen/ids 13007)
        run (gen/cyrillic rng 6)
        query (str (gen/hiragana rng 6)
                   "\n\nResearch:\n1. "
                   (gen/greek rng 4))
        processor (gen/greek rng 6)
        language (gen/cyrillic rng 6)
        provider (gen/cyrillic rng 6)
        item (pending/pending {:run_id run
                               :query query
                               :processor processor
                               :language language
                               :provider provider})]
    (is (= language (pending/language item))
        "Pending language did not match provided value")))

(deftest the-pending-serializes-correctly
  (let [rng (gen/ids 13009)
        run (gen/cyrillic rng 6)
        query (str (gen/hiragana rng 6)
                   "\n\nResearch:\n1. "
                   (gen/greek rng 4))
        processor (gen/greek rng 6)
        language (gen/cyrillic rng 6)
        provider (gen/cyrillic rng 6)
        item (pending/pending {:run_id run
                               :query query
                               :processor processor
                               :language language
                               :provider provider})
        data (pending/data item)
        brief (:brief data)
        items (:items brief)
        node (first items)]
    (is (and (contains? data :run_id)
             (contains? data :processor)
             (contains? data :language)
             (contains? data :brief)
             (contains? brief :topic)
             (contains? brief :items)
             (contains? node :text)
             (contains? node :items)
             (not (contains? brief :text))
             (not (contains? data :query)))
        "Pending serialize did not include brief or still included query")))

(deftest ^{:doc "Ensure pending nested query items are parsed."}
  the-pending-parses-nested-query-items
  (let [rng (gen/ids 13010)
        run (gen/cyrillic rng 6)
        head (gen/greek rng 5)
        inner (gen/armenian rng 5)
        tail (gen/hiragana rng 5)
        pad (apply str (repeat 4 " "))
        query (str (gen/cyrillic rng 6)
                   "\n\nResearch:\n1. "
                   head
                   "\n"
                   pad
                   "1. "
                   inner
                   "\n2. "
                   tail)
        item (pending/pending {:run_id run
                               :query query
                               :processor (gen/greek rng 6)
                               :language (gen/cyrillic rng 6)
                               :provider (gen/cyrillic rng 6)})
        brief (pending/brief item)
        items (:items brief)
        node (first items)
        peer (second items)
        ok (and (= head (:text node))
                (= inner (:text (first (:items node))))
                (= tail (:text peer)))]
    (is ok "Pending nested query items were not parsed")))

(deftest the-pending-deserializes-correctly
  (let [rng (gen/ids 13011)
        run (gen/cyrillic rng 6)
        query (gen/hiragana rng 6)
        processor (gen/greek rng 6)
        language (gen/cyrillic rng 6)
        provider (gen/cyrillic rng 6)
        item (pending/pending {:run_id run
                               :query query
                               :processor processor
                               :language language
                               :provider provider})]
    (is (= run (pending/id item))
        "Pending deserialize did not restore identifier")))

(deftest the-pending-returns-provider
  (let [rng (gen/ids 13013)
        run (gen/cyrillic rng 6)
        query (gen/hiragana rng 6)
        processor (gen/greek rng 6)
        language (gen/cyrillic rng 6)
        name (gen/cyrillic rng 6)
        item (pending/pending {:run_id run
                               :query query
                               :processor processor
                               :language language
                               :provider name})]
    (is (= name (pending/provider item))
        "Pending provider did not match provided value")))

(deftest the-pending-prefers-explicit-topic-in-brief
  (let [rng (gen/ids 13017)
        run (gen/cyrillic rng 6)
        mark (gen/hiragana rng 6)
        query (str (gen/greek rng 6)
                   "\n\nResearch:\n1. "
                   (gen/armenian rng 4))
        item (pending/pending {:run_id run
                               :query query
                               :processor (gen/greek rng 6)
                               :language (gen/cyrillic rng 6)
                               :provider (gen/cyrillic rng 6)
                               :topic mark})
        brief (pending/brief item)
        topic (:topic brief)]
    (is (= mark topic)
        "Pending brief did not use explicit topic")))

(deftest the-pending-serializes-provider
  (let [rng (gen/ids 13015)
        run (gen/cyrillic rng 6)
        query (gen/hiragana rng 6)
        processor (gen/greek rng 6)
        language (gen/cyrillic rng 6)
        name (gen/hiragana rng 6)
        item (pending/pending {:run_id run
                               :query query
                               :processor processor
                               :language language
                               :provider name})
        data (pending/data item)]
    (is (= name (:provider data))
        "Pending serialize did not include provider")))
