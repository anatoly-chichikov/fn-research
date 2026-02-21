(ns research.domain.task
  (:refer-clojure :exclude [format])
  (:require [clojure.string :as str]
            [research.domain.brief :as brief]
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

(defrecord ResearchRun [id brief data result]
  Tasked
  (id [_] id)
  (brief [_] brief)
  (query [_]
    (brief/render brief (:language data)))
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
