(ns research.domain.brief
  (:require [clojure.string :as str]))

(defn node
  "Normalize brief item map."
  [item]
  (let [text (str (or (:text item) item))
        list (or (:items item) [])
        list (mapv node list)
        text (str/trim text)
        list (vec (remove
                   (fn [item]
                     (and (str/blank? (:text item))
                          (empty? (:items item))))
                   list))]
    {:text text
     :items list}))

(defn point
  "Parse list line into depth item."
  [line]
  (let [line (str/replace (str line) #"\t" " ")
        trim (str/triml line)
        pad (- (count line) (count trim))
        num (re-find #"^(\d+(?:\.\d+)*)[.)]?\s+(.+)$" trim)
        bul (re-find #"^[*+-]\s+(.+)$" trim)
        text (cond
               num (nth num 2)
               bul (second bul)
               :else nil)
        base (cond
               num (count (str/split (nth num 1) #"\."))
               bul (inc (quot pad 2))
               :else nil)
        depth (cond
                num (if (pos? pad) (inc (quot pad 4)) base)
                bul base
                :else nil)
        depth (if depth (max 1 (min depth 3)) nil)
        text (str/trim (or text ""))]
    (if (and depth (not (str/blank? text)))
      {:depth depth
       :text text}
      nil)))

(defn scan
  "Parse list lines into flat items."
  [lines]
  (loop [list [] tail lines]
    (if (seq tail)
      (let [raw (first tail)
            item (point raw)
            line (str/trim (str raw))
            list (cond
                   (str/blank? line) list
                   item (conj list item)
                   (seq list)
                   (conj (vec (butlast list))
                         (update (last list) :text str " " line))
                   :else list)]
        (recur list (rest tail)))
      list)))

(defn place
  "Insert item at depth."
  [items depth item]
  (let [depth (if (and (> depth 1) (empty? items)) 1 depth)]
    (if (= depth 1)
      (conj items item)
      (let [last (last items)
            head (vec (butlast items))
            last (if last last {:text ""
                                :items []})
            tail (update last :items place (dec depth) item)]
        (conj head tail)))))

(defn nest
  "Nest flat items into tree."
  [list]
  (loop [items [] tail list]
    (if (seq tail)
      (let [item (first tail)
            node {:text (:text item)
                  :items []}
            items (place items (:depth item) node)]
        (recur items (rest tail)))
      items)))

(defn lines
  "Render nested items into markdown list."
  [items depth]
  (let [pad (apply str (repeat (* 4 depth) " "))]
    (loop [idx 0 list []]
      (if (< idx (count items))
        (let [item (nth items idx)
              text (str/trim (str (or (:text item) "")))
              nest (or (:items item) [])
              rows (lines nest (inc depth))
              line (if (str/blank? text)
                     nil
                     (str pad (inc idx) ". " text))
              list (cond
                     (and (nil? line) (seq rows)) (into list rows)
                     (nil? line) list
                     (seq rows) (into (conj list line) rows)
                     :else (conj list line))]
          (recur (inc idx) list))
        list))))

(defn render
  "Render brief into query text."
  [brief language]
  (let [lang (str/trim (str (or language "")))
        lead (if (str/blank? lang) "" (str "Язык ответа: " lang "."))
        topic (str (or (:topic brief) ""))
        items (or (:items brief) [])
        items (mapv node items)
        rows (lines items 0)
        tail (str/join "\n" rows)
        body (cond
               (seq rows)
               (if (str/blank? topic)
                 (str "Research:\n" tail)
                 (str topic "\n\nResearch:\n" tail))
               :else topic)
        note (cond
               (and (seq lead) (seq body)) (str lead "\n\n" body)
               (seq lead) lead
               :else body)]
    note))
