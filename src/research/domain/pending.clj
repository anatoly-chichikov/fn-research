(ns research.domain.pending
  (:require [clojure.string :as str]
            [research.domain.brief :as brief]))

(defprotocol Pendinged
  "Object with pending run details."
  (id [item] "Return run identifier.")
  (brief [item] "Return brief details.")
  (query [item] "Return research query.")
  (processor [item] "Return processor name.")
  (language [item] "Return research language.")
  (provider [item] "Return provider name.")
  (data [item] "Return map representation."))

(defrecord PendingRun [id brief data]
  Pendinged
  (id [_] id)
  (brief [_] brief)
  (query [_]
    (brief/render brief (:language data)))
  (processor [_] (:processor data))
  (language [_] (:language data))
  (provider [_] (:provider data))
  (data [_] {:run_id id
             :processor (:processor data)
             :language (:language data)
             :provider (:provider data)
             :brief (dissoc brief :text)}))

(defn pending
  "Create pending run from map."
  [item]
  (let [entry (:brief item)
        query (or (:text entry) (:query item) "")
        rows (str/split-lines (str query))
        label "Research:"
        spot (first (keep-indexed
                     (fn [idx line]
                       (when (= label (str/trim line)) idx))
                     rows))
        edge (first (keep-indexed
                     (fn [idx line]
                       (when (brief/point line) idx))
                     rows))
        cut (if (some? spot) spot edge)
        head (vec (if (some? cut) (take cut rows) rows))
        tail (if (some? cut)
               (drop (if (some? spot) (inc spot) cut) rows)
               [])
        list (brief/nest (brief/scan tail))
        top (reduce
             (fn [text line]
               (if (str/blank? (str/trim line)) text (str/trim line)))
             ""
             head)
        topic (or (:topic entry) top "")
        items (if (seq (:items entry)) (:items entry) list)
        items (mapv brief/node items)
        brief {:topic topic
               :items items}]
    (->PendingRun
     (:run_id item)
     brief
     {:processor (:processor item)
      :language (:language item)
      :provider (or (:provider item) "parallel")})))
