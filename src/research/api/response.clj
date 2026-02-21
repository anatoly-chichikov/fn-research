(ns research.api.response
  (:require [clojure.string :as str]
            [research.api.link :as link]
            [research.domain.result :as result]))

(defprotocol Responded
  "Object representing API response."
  (id [item] "Return run identifier.")
  (cost [item] "Return total cost.")
  (raw [item] "Return raw response map.")
  (completed [item] "Return true when completed.")
  (failed [item] "Return true when failed.")
  (text [item] "Return output markdown.")
  (sources [item] "Return source list."))

(defn clean
  "Remove utm params from URL."
  [text]
  (link/clean (link/make) text))

(defn strip
  "Strip tracking URLs from output text."
  [text]
  (link/strip (link/make) text))

(defn domain
  "Extract domain from URL string."
  [text]
  (link/domain (link/make) text))

(defrecord Response [id status text data]
  Responded
  (id [_] id)
  (cost [_] (:cost data))
  (raw [_] (:raw data))
  (completed [_] (= status "completed"))
  (failed [_] (= status "failed"))
  (text [_] text)
  (sources [_] (let [policy (or (:link data) (link/make))
                     state (reduce
                            (fn [state field]
                              (let [items (get field :citations [])]
                                (reduce
                                 (fn [state cite]
                                   (let [url (or (:url cite) "")
                                         url (if (str/blank? url)
                                               ""
                                               (link/clean policy url))
                                         flag (and (not (str/blank? url))
                                                   (not (contains?
                                                         (:seen state)
                                                         url)))]
                                     (if flag
                                       (let [ex (get cite :excerpts [])
                                             note (if (seq ex) (first ex) "")
                                             head (or (:title cite)
                                                      (link/domain policy url))
                                             item (result/->CitationSource
                                                   head
                                                   url
                                                   note)]
                                         {:seen (conj (:seen state) url)
                                          :list (conj (:list state) item)})
                                       state)))
                                 state
                                 items)))
                            {:seen #{}
                             :list []}
                            (or (:basis data) []))]
                 (:list state))))

(defn response
  "Create response from map."
  [item]
  (let [policy (or (:link item) (link/make))
        text (link/strip policy (or (:output item) ""))
        base {:basis (or (:basis item) [])
              :cost (or (:cost item) 0.0)
              :raw (or (:raw item) {})
              :link policy}]
    (->Response (:id item) (:status item) text base)))
