(ns research.main.seed
  (:require [research.domain.session :as session]
            [research.domain.task :as task]
            [research.storage.repository :as repo])
  (:import (java.util UUID)))

(defn seed
  "Create session."
  [data topic query processor language provider]
  (let [repo (repo/repo data)
        id (str (UUID/randomUUID))
        map {:id id
             :topic topic
             :tasks []
             :created (task/format (task/now))
             :query query
             :processor processor
             :language language
             :provider provider}
        value (session/session map)]
    (repo/append repo value)
    (println (str "Created session: " (subs id 0 8)))
    (println (str "Topic: " topic))
    (subs id 0 8)))
