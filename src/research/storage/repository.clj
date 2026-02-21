(ns research.storage.repository
  (:refer-clojure :exclude [find load update])
  (:require [research.domain.session :as session]
            [research.storage.repository.read :as read]
            [research.storage.repository.write :as write])
  (:import (java.io RandomAccessFile)
           (java.nio.file Files)
           (java.nio.file.attribute FileAttribute)
           (java.util Optional)
           (java.util.concurrent.locks ReentrantLock)))

(defprotocol Loadable
  "Object that can load sessions."
  (load [item] "Return session list."))

(defprotocol Savable
  "Object that can save sessions."
  (save [item list] "Persist session list."))

(defprotocol Mutable
  "Object that can update sessions."
  (append [item value] "Append session.")
  (find [item value] "Find session by id.")
  (update [item value] "Update session by id."))

(defn- locked
  "Execute function under file lock."
  [guard root func]
  (.lock guard)
  (try
    (let [_ (Files/createDirectories root (make-array FileAttribute 0))
          path (.resolve root ".repo.lock")
          file (RandomAccessFile. (.toFile path) "rw")
          channel (.getChannel file)]
      (try
        (let [fl (.lock channel)]
          (try
            (func)
            (finally
              (.release fl))))
        (finally
          (.close file))))
    (finally
      (.unlock guard))))

(defrecord Repository [root guard]
  Loadable
  (load [_]
    (read/items root))
  Savable
  (save [_ items]
    (write/store root items))
  Mutable
  (append [_ value]
    (locked guard root
            (fn []
              (let [items (read/items root)
                    store (conj items value)]
                (write/store root store)))))
  (find [item value]
    (let [items (load item)
          pick (first (filter #(= (session/id %) value) items))]
      (if pick (Optional/of pick) (Optional/empty))))
  (update [_ value]
    (locked guard root
            (fn []
              (let [items (read/items root)
                    store (mapv (fn [node]
                                  (if (= (session/id node)
                                         (session/id value))
                                    value
                                    node))
                                items)]
                (write/store root store))))))

(defn repo
  "Create repository from output path."
  [item]
  (->Repository item (ReentrantLock.)))
