(ns research.domain.session
  (:refer-clojure :exclude [extend format])
  (:require [research.domain.pending :as pending]
            [research.domain.task :as task])
  (:import (java.time LocalDateTime)
           (java.time.format DateTimeFormatter)
           (java.util Optional UUID)))

(defprotocol Sessioned
  "Object representing research session."
  (id [item] "Return session identifier.")
  (topic [item] "Return session topic.")
  (tasks [item] "Return task list.")
  (created [item] "Return creation time.")
  (pending [item] "Return pending run.")
  (query [item] "Return research query.")
  (processor [item] "Return processor name.")
  (language [item] "Return research language.")
  (provider [item] "Return provider name.")
  (extend [item value] "Return new session with appended task.")
  (start [item value] "Return session with pending run.")
  (reset [item] "Return session without pending run.")
  (reconfigure [item opts] "Return session with updated research parameters.")
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

(defrecord ResearchSession [id topic tasks data]
  Sessioned
  (id [_] id)
  (topic [_] topic)
  (tasks [_] tasks)
  (created [_] (:created data))
  (pending [_] (:pending data))
  (query [_] (:query data))
  (processor [_] (:processor data))
  (language [_] (:language data))
  (provider [_] (:provider data))
  (extend [_ value]
    (->ResearchSession
     id
     topic
     (conj tasks value)
     (assoc data :pending (Optional/empty))))
  (start [_ value]
    (->ResearchSession
     id
     topic
     tasks
     (assoc data :pending (Optional/of value))))
  (reset [_]
    (->ResearchSession id topic tasks (assoc data :pending (Optional/empty))))
  (reconfigure [_ opts]
    (->ResearchSession id topic tasks (merge data opts)))
  (data [_] (let [base {:id id
                        :topic topic
                        :tasks (mapv task/data tasks)
                        :created (format (:created data))}
                  base (cond-> base
                         (seq (:query data))
                         (assoc :query (:query data))
                         (seq (:processor data))
                         (assoc :processor (:processor data))
                         (seq (:language data))
                         (assoc :language (:language data))
                         (seq (:provider data))
                         (assoc :provider (:provider data)))
                  hold (:pending data)
                  pack (if (.isPresent hold)
                         (assoc base :pending (pending/data (.get hold)))
                         base)]
              pack)))

(defn session
  "Create session from map."
  [item]
  (let [list (mapv task/task (:tasks item))
        time (parse (:created item))
        hold (if (:pending item)
               (Optional/of (pending/pending (:pending item)))
               (Optional/empty))
        data {:created time
              :pending hold
              :query (or (:query item) "")
              :processor (or (:processor item) "")
              :language (or (:language item) "")
              :provider (or (:provider item) "")}
        code (or (:id item) (str (UUID/randomUUID)))]
    (->ResearchSession code (:topic item) list data)))
