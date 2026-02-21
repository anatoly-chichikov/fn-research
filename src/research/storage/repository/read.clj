(ns research.storage.repository.read
  (:require [clojure.edn :as edn]
            [clojure.pprint :as pprint]
            [clojure.string :as str]
            [research.domain.session :as session])
  (:import (java.nio.file Files LinkOption StandardCopyOption)
           (java.nio.file.attribute FileAttribute)))

(defn items
  "Return sessions from output folder."
  [root]
  (let [opts (make-array LinkOption 0)
        list (if (Files/exists root opts)
               (with-open [stream (Files/newDirectoryStream root)]
                 (reduce
                  (fn [list path]
                    (let [file (.resolve path "session.edn")
                          dir (Files/isDirectory path opts)
                          hit (Files/exists file opts)
                          name (if dir (str (.getFileName path)) "")
                          rule (re-pattern
                                (str "(\\d{4}-\\d{2}-\\d{2})"
                                     "_(.+)_([A-Za-z0-9]{8})"))
                          mark (if dir (re-matches rule name) nil)]
                      (cond
                        (and dir hit)
                        (let [text (slurp (.toFile file) :encoding "UTF-8")
                              data (edn/read-string text)
                              tasks (:tasks data)
                              hold (:pending data)
                              flag (or (some #(or (contains? % :query)
                                                  (contains? % :result)
                                                  (not (contains? % :brief))
                                                  (contains?
                                                   (get % :brief {})
                                                   :text))
                                             tasks)
                                       (and hold
                                            (or (contains? hold :query)
                                                (contains? hold :result)
                                                (not (contains? hold :brief))
                                                (contains?
                                                 (get hold :brief {})
                                                 :text))))
                              items (vec (or tasks []))
                              data (assoc data :tasks items :pending hold)
                              item (session/session data)
                              note (if flag
                                     (with-out-str
                                       (pprint/pprint (session/data item)))
                                     "")
                              _ (when flag
                                  (let [dir (.getParent file)
                                        tmp (Files/createTempFile
                                             dir "session" ".tmp"
                                             (make-array FileAttribute 0))]
                                    (try
                                      (spit (.toFile tmp) note
                                            :encoding "UTF-8")
                                      (Files/move
                                       tmp file
                                       (into-array
                                        java.nio.file.CopyOption
                                        [StandardCopyOption/ATOMIC_MOVE]))
                                      (catch Exception exc
                                        (Files/deleteIfExists tmp)
                                        (throw exc)))))]
                          (conj list item))
                        mark
                        (let [date (nth mark 1)
                              slug (nth mark 2)
                              code (nth mark 3)
                              time (str date "T00:00:00")
                              id (str code "-migrated")
                              names
                              (with-open [files
                                          (Files/newDirectoryStream path)]
                                (let [list (reduce
                                            (fn [list file]
                                              (let [name
                                                    (str
                                                     (.getFileName file))
                                                    head
                                                    (str/starts-with?
                                                     name
                                                     "response-")
                                                    tail
                                                    (str/ends-with?
                                                     name
                                                     ".json")
                                                    mark
                                                    (and head tail)]
                                                (if mark
                                                  (conj list name)
                                                  list)))
                                            []
                                            files)]
                                  (vec (sort list))))
                              tasks (map-indexed
                                     (fn [index name]
                                       (let [size (count name)
                                             tag (subs name 9 (- size 5))
                                             item (str code "-" tag "-")
                                             item (str item index)]
                                         {:id item
                                          :status "completed"
                                          :service (if (= tag "xai")
                                                     "x.ai"
                                                     (str tag ".ai"))
                                          :created time}))
                                     names)
                              data {:id id
                                    :topic slug
                                    :tasks tasks
                                    :created time}
                              item (session/session data)
                              text (with-out-str
                                     (pprint/pprint (session/data item)))]
                          (let [dir (.getParent file)
                                tmp (Files/createTempFile
                                     dir "session" ".tmp"
                                     (make-array FileAttribute 0))]
                            (try
                              (spit (.toFile tmp) text :encoding "UTF-8")
                              (Files/move
                               tmp file
                               (into-array
                                java.nio.file.CopyOption
                                [StandardCopyOption/ATOMIC_MOVE]))
                              (catch Exception exc
                                (Files/deleteIfExists tmp)
                                (throw exc))))
                          (conj list item))
                        :else list)))
                  []
                  stream))
               [])]
    (vec (sort-by session/created list))))
