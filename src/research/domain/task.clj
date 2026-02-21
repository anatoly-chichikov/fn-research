(ns research.domain.task
  (:refer-clojure :exclude [format])
  (:require [clojure.string :as str]
            [research.domain.result :as result])
  (:import (java.time LocalDateTime)
           (java.time.format DateTimeFormatter)
           (java.util Optional UUID)))

(defprotocol Tasked
  "Object representing a research task."
  (id [item] "Return task identifier.")
  (brief [item] "Return brief details.")
  (query [item] "Return research query.")
  (status [item] "Return task status.")
  (report [item] "Return task result object.")
  (language [item] "Return task language.")
  (provider [item] "Return task provider.")
  (created [item] "Return creation time.")
  (completed [item] "Return completion time.")
  (finish [item value] "Return task marked as completed.")
  (data [item] "Return map representation."))

(defn now
  "Return current local datetime."
  []
  (LocalDateTime/now))

(defn parse
  "Parse ISO datetime string into LocalDateTime."
  [text]
  (LocalDateTime/parse text))

(defn format
  "Format LocalDateTime into ISO string."
  [time]
  (.format time DateTimeFormatter/ISO_LOCAL_DATE_TIME))

(defn- node
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

(defn- point
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

(defn- scan
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

(defn- place
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

(defn- nest
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

(defn- lines
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

(defn- render
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

(defrecord ResearchRun [id brief data result]
  Tasked
  (id [_] id)
  (brief [_] brief)
  (query [_]
    (render brief (:language data)))
  (status [_] (:status data))
  (report [_] result)
  (language [_] (:language data))
  (provider [_] (:service data))
  (created [_] (:created data))
  (completed [_] (:completed data))
  (finish [_ value]
    (->ResearchRun
     id
     brief
     (assoc data :status "completed" :completed (Optional/of (now)))
     value))
  (data [_] (let [brief (dissoc brief :text)
                  base {:id id
                        :status (:status data)
                        :language (:language data)
                        :service (:service data)
                        :processor (:processor data)
                        :brief brief
                        :created (format (:created data))}
                  done (:completed data)
                  ready (if (.isPresent done)
                          (assoc base :completed (format (.get done)))
                          base)
                  proc (:processor data)
                  ready (if (str/blank? (str proc))
                          (dissoc ready :processor)
                          ready)]
              ready)))

(defn task
  "Create task from map."
  [item]
  (let [text (or (:language item) "русский")
        name (or (:service item) "parallel.ai")
        parts (str/split name #"\.")
        name (if (and (str/ends-with? name ".ai")
                      (= (first parts) "xai")
                      (not= name "x.ai"))
               "x.ai"
               name)
        time (parse (:created item))
        done (if (:completed item)
               (Optional/of (parse (:completed item)))
               (Optional/empty))
        entry (:brief item)
        query (or (:text entry) (:query item) "")
        rows (str/split-lines (str query))
        label "Research:"
        spot (first (keep-indexed
                     (fn [idx line]
                       (when (= label (str/trim line)) idx))
                     rows))
        edge (first (keep-indexed
                     (fn [idx line]
                       (when (point line) idx))
                     rows))
        cut (if (some? spot) spot edge)
        head (vec (if (some? cut) (take cut rows) rows))
        tail (if (some? cut)
               (drop (if (some? spot) (inc spot) cut) rows)
               [])
        list (nest (scan tail))
        top (reduce
             (fn [text line]
               (if (str/blank? (str/trim line)) text (str/trim line)))
             ""
             head)
        topic (or (:topic entry) (:topic item) top "")
        items (if (seq (:items entry)) (:items entry) list)
        items (mapv node items)
        brief {:topic topic
               :items items}
        data {:status (:status item)
              :language text
              :service name
              :processor (:processor item)
              :created time
              :completed done}
        raw (:result item)
        value (result/result raw)
        code (or (:id item) (str (UUID/randomUUID)))]
    (->ResearchRun code brief data value)))
