(ns research.api.xai.brief
  (:require [clojure.string :as str]))

(defprotocol Briefed
  "Object that can parse research brief."
  (parts [item text] "Return brief parts."))

(defrecord Brief [mark]
  Briefed
  (parts [_ text]
    (let [lines (str/split-lines (str text))
          spot (first (keep-indexed
                       (fn [idx line]
                         (when (= mark (str/trim line)) idx))
                       lines))
          head (vec (if (some? spot) (take spot lines) lines))
          tail (if (some? spot) (drop (inc spot) lines) [])
          items (loop [list [] chunk tail]
                  (if (seq chunk)
                    (let [raw (str (first chunk))
                          tabs (count (take-while #(= \tab %) raw))
                          row (str/replace raw #"\t" " ")
                          trim (str/triml row)
                          pad (- (count row) (count trim))
                          num (re-find #"^(\d+(?:\.\d+)*)[.)]?\s+(.+)$" trim)
                          bul (re-find #"^[*+-]\s+(.+)$" trim)
                          plain (and (nil? num)
                                     (nil? bul)
                                     (not (str/blank? trim))
                                     (or (pos? tabs) (zero? pad)))
                          text (cond
                                 num (nth num 2)
                                 bul (second bul)
                                 plain trim
                                 :else nil)
                          base (cond
                                 num (count (str/split (nth num 1) #"\."))
                                 bul (inc (quot pad 2))
                                 plain (inc tabs)
                                 :else nil)
                          depth (cond
                                  num (if (pos? pad) (inc (quot pad 4)) base)
                                  bul base
                                  plain base
                                  :else nil)
                          depth (if depth (max 1 depth) nil)
                          depth (if depth (min depth 3) nil)
                          text (str/trim (str (or text "")))
                          item (when (and depth (not (str/blank? text)))
                                 {:depth depth
                                  :text text})
                          line (str/trim row)
                          list (cond
                                 (str/blank? line) list
                                 item (conj list item)
                                 (seq list) (conj (vec (butlast list))
                                                  (update (last list)
                                                          :text
                                                          str
                                                          " "
                                                          line))
                                 :else list)]
                      (recur list (rest chunk)))
                    list))
          top (reduce
               (fn [text line]
                 (if (str/blank? (str/trim line)) text (str/trim line)))
               ""
               head)]
      {:head head
       :items items
       :top top})))

(defn make
  "Return brief parser."
  []
  (->Brief "Research:"))
