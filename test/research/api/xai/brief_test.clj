(ns research.api.xai.brief-test
  (:require [clojure.test :refer [deftest is]]
            [research.api.xai.brief :as brief]
            [research.test.ids :as gen]))

(deftest the-brief-parses-items
  (let [rng (gen/ids 18307)
        head (gen/cyrillic rng 5)
        left (gen/greek rng 4)
        right (gen/armenian rng 4)
        text (str head "\n\nResearch:\n1. " left "\n2. " right)
        item (brief/make)
        info (brief/parts item text)
        items (:items info)
        expect [{:depth 1
                 :text left}
                {:depth 1
                 :text right}]]
    (is (= expect items) "brief did not parse items")))

(deftest the-brief-parses-nested-items
  (let [rng (gen/ids 18308)
        head (gen/cyrillic rng 5)
        alpha (gen/greek rng 4)
        beta (gen/armenian rng 4)
        gamma (gen/arabic rng 4)
        delta (gen/hebrew rng 4)
        pad (apply str (repeat 4 " "))
        deep (str pad pad)
        text (str head
                  "\n\nResearch:\n1. "
                  alpha
                  "\n"
                  pad
                  "1. "
                  beta
                  "\n"
                  deep
                  "1. "
                  gamma
                  "\n2. "
                  delta)
        item (brief/make)
        info (brief/parts item text)
        items (:items info)
        expect [{:depth 1
                 :text alpha}
                {:depth 2
                 :text beta}
                {:depth 3
                 :text gamma}
                {:depth 1
                 :text delta}]]
    (is (= expect items) "nested brief items were not parsed")))

(deftest the-brief-parses-tab-indented-items
  (let [rng (gen/ids 18309)
        head (gen/cyrillic rng 5)
        left (gen/greek rng 4)
        child (gen/armenian rng 4)
        right (gen/hebrew rng 4)
        text (str head
                  "\n\nResearch:\n"
                  left
                  "\n\t"
                  child
                  "\n"
                  right)
        item (brief/make)
        info (brief/parts item text)
        items (:items info)
        expect [{:depth 1
                 :text left}
                {:depth 2
                 :text child}
                {:depth 1
                 :text right}]]
    (is (= expect items) "tab-indented brief items were not parsed")))

(deftest the-brief-parses-double-tab-items
  (let [rng (gen/ids 18310)
        head (gen/cyrillic rng 5)
        alpha (gen/greek rng 4)
        beta (gen/armenian rng 4)
        gamma (gen/arabic rng 4)
        delta (gen/hebrew rng 4)
        text (str head
                  "\n\nResearch:\n"
                  alpha
                  "\n\t"
                  beta
                  "\n\t\t"
                  gamma
                  "\n"
                  delta)
        item (brief/make)
        info (brief/parts item text)
        items (:items info)
        expect [{:depth 1
                 :text alpha}
                {:depth 2
                 :text beta}
                {:depth 3
                 :text gamma}
                {:depth 1
                 :text delta}]]
    (is (= expect items) "double-tab brief items were not parsed")))
