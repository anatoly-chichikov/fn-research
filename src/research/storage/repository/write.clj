(ns research.storage.repository.write
  (:require [clojure.pprint :as pprint]
            [research.domain.session :as session]
            [research.storage.organizer :as organizer])
  (:import (java.nio.file Files StandardCopyOption)
           (java.nio.file.attribute FileAttribute)))

(defn store
  "Persist session list into output folder."
  [root items]
  (let [org (organizer/organizer root)]
    (doseq [item items]
      (let [name (organizer/name
                  org
                  (session/created item)
                  (session/topic item)
                  (session/id item))
            base (.resolve root name)
            _ (Files/createDirectories base (make-array FileAttribute 0))
            path (.resolve base "session.edn")
            text (with-out-str (pprint/pprint (session/data item)))
            temp (Files/createTempFile base "session" ".tmp"
                                       (make-array FileAttribute 0))]
        (try
          (spit (.toFile temp) text :encoding "UTF-8")
          (Files/move temp path
                      (into-array java.nio.file.CopyOption
                                  [StandardCopyOption/ATOMIC_MOVE]))
          (catch Exception exc
            (Files/deleteIfExists temp)
            (throw exc)))))))
