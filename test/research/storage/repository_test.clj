(ns research.storage.repository-test
  (:require [clojure.test :refer [deftest is]]
            [research.domain.result :as result]
            [research.domain.session :as session]
            [research.domain.task :as task]
            [research.storage.repository :as repo]
            [research.test.ids :as gen])
  (:import (java.nio.file Files)
           (java.nio.file.attribute FileAttribute)))

(deftest the-repository-returns-empty-for-empty-folder
  (let [rng (gen/ids 24001)
        root (Files/createTempDirectory "repo"
                                        (make-array FileAttribute 0))
        mark (gen/ascii rng 4)
        name (str mark "-" (gen/uuid rng))
        path (.resolve root name)
        _ (Files/createDirectories path (make-array FileAttribute 0))
        item (repo/repo root)]
    (is (= 0 (count (repo/load item)))
        "Load did not return empty list for empty folder")))

(deftest the-repository-saves-and-loads-session
  (let [rng (gen/ids 24003)
        root (Files/createTempDirectory "repo"
                                        (make-array FileAttribute 0))
        item (repo/repo root)
        topic (gen/cyrillic rng 6)
        ident (gen/uuid rng)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        entry (session/session {:id ident
                                :topic topic
                                :tasks []
                                :created time})]
    (repo/save item [entry])
    (is (= topic (session/topic (first (repo/load item))))
        "Loaded session topic did not match saved")))

(deftest the-repository-append-adds-session
  (let [rng (gen/ids 24005)
        root (Files/createTempDirectory "repo"
                                        (make-array FileAttribute 0))
        item (repo/repo root)
        topic (gen/cyrillic rng 6)
        label (gen/cyrillic rng 6)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        alpha (session/session {:id (gen/uuid rng)
                                :topic topic
                                :tasks []
                                :created time})
        beta (session/session {:id (gen/uuid rng)
                               :topic label
                               :tasks []
                               :created time})]
    (repo/append item alpha)
    (repo/append item beta)
    (is (= 2 (count (repo/load item)))
        "Repository did not contain two sessions after append")))

(deftest the-repository-find-returns-matching-session
  (let [rng (gen/ids 24007)
        root (Files/createTempDirectory "repo"
                                        (make-array FileAttribute 0))
        item (repo/repo root)
        topic (gen/cyrillic rng 6)
        ident (gen/uuid rng)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        entry (session/session {:id ident
                                :topic topic
                                :tasks []
                                :created time})]
    (repo/append item entry)
    (let [hit (repo/find item ident)]
      (is (= topic (session/topic (.get hit)))
          "Found session topic did not match"))))

(deftest the-repository-find-returns-empty-for-missing
  (let [rng (gen/ids 24009)
        root (Files/createTempDirectory "repo"
                                        (make-array FileAttribute 0))
        item (repo/repo root)
        code (gen/cyrillic rng 6)
        hit (repo/find item code)]
    (is (not (.isPresent hit)) "Find returned value for missing ID")))

(deftest the-repository-update-modifies-session
  (let [rng (gen/ids 24011)
        root (Files/createTempDirectory "repo"
                                        (make-array FileAttribute 0))
        item (repo/repo root)
        topic (gen/cyrillic rng 6)
        ident (gen/uuid rng)
        day (inc (.nextInt rng 8))
        hour (inc (.nextInt rng 8))
        time (str "2026-01-0" day "T0" hour ":00:00")
        entry (session/session {:id ident
                                :topic topic
                                :tasks []
                                :created time})]
    (repo/append item entry)
    (let [query (gen/cyrillic rng 6)
          status (gen/cyrillic rng 6)
          language (gen/cyrillic rng 5)
          service (gen/cyrillic rng 4)
          summary (gen/cyrillic rng 6)
          value (result/->ResearchReport summary [])
          task (task/task {:id (gen/uuid rng)
                           :query query
                           :status status
                           :language language
                           :service service
                           :result (result/data value)
                           :created time})
          revision (session/extend entry task)]
      (repo/update item revision)
      (let [hit (repo/find item ident)]
        (is (= 1 (count (session/tasks (.get hit))))
            "Updated session did not contain task")))))

(deftest ^{:doc "Migrates legacy session folders"}
  the-repository-migrates-legacy-folders
  (let [rng (gen/ids 24013)
        root (Files/createTempDirectory "repo"
                                        (make-array FileAttribute 0))
        date (str "2026-01-0" (inc (.nextInt rng 8)))
        slug (gen/ascii rng 6)
        code (subs (gen/uuid rng) 0 8)
        name (str date "_" slug "_" code)
        path (.resolve root name)
        _ (Files/createDirectories path (make-array FileAttribute 0))
        tag (gen/ascii rng 4)
        response (.resolve path (str "response-" tag ".json"))
        _ (spit (.toFile response) "{}" :encoding "UTF-8")
        repo (repo/repo root)
        _ (repo/load repo)
        file (.resolve path "session.edn")
        flag (.exists (.toFile file))]
    (is flag "Migration did not create session edn")))

(deftest ^{:doc "Builds tasks from response files"}
  the-repository-builds-tasks-from-responses
  (let [rng (gen/ids 24015)
        root (Files/createTempDirectory "repo"
                                        (make-array FileAttribute 0))
        date (str "2026-01-0" (inc (.nextInt rng 8)))
        slug (gen/ascii rng 6)
        code (subs (gen/uuid rng) 0 8)
        name (str date "_" slug "_" code)
        path (.resolve root name)
        _ (Files/createDirectories path (make-array FileAttribute 0))
        alpha (gen/ascii rng 4)
        beta (gen/ascii rng 4)
        left (.resolve path (str "response-" alpha ".json"))
        right (.resolve path (str "response-" beta ".json"))
        _ (spit (.toFile left) "{}" :encoding "UTF-8")
        _ (spit (.toFile right) "{}" :encoding "UTF-8")
        repo (repo/repo root)
        list (repo/load repo)
        item (first list)
        size (count (session/tasks item))]
    (is (= 2 size) "Migration did not build tasks from responses")))

(deftest ^{:doc "Concurrent appends preserve all sessions"}
  the-repository-survives-concurrent-appends
  (let [rng (gen/ids 24019)
        root (Files/createTempDirectory "repo"
                                        (make-array FileAttribute 0))
        item (repo/repo root)
        size 8
        time "2026-01-05T06:00:00"
        list (mapv (fn [_]
                     (session/session {:id (gen/uuid rng)
                                       :topic (gen/cyrillic rng 6)
                                       :tasks []
                                       :created time}))
                   (range size))
        pool (java.util.concurrent.Executors/newFixedThreadPool 4)
        latch (java.util.concurrent.CountDownLatch. size)
        _ (doseq [entry list]
            (.submit pool
                     ^Runnable
                     (fn []
                       (try
                         (repo/append item entry)
                         (finally
                           (.countDown latch))))))]
    (.await latch 30 java.util.concurrent.TimeUnit/SECONDS)
    (.shutdown pool)
    (is (= size (count (repo/load item)))
        "Concurrent appends lost sessions")))

(deftest ^{:doc "Strips query from session edn"}
  the-repository-strips-query-from-session-edn
  (let [rng (gen/ids 24017)
        root (Files/createTempDirectory "repo"
                                        (make-array FileAttribute 0))
        date (str "2026-01-0" (inc (.nextInt rng 8)))
        slug (gen/ascii rng 6)
        code (subs (gen/uuid rng) 0 8)
        name (str date "_" slug "_" code)
        path (.resolve root name)
        _ (Files/createDirectories path (make-array FileAttribute 0))
        tag (gen/ascii rng 4)
        note (gen/cyrillic rng 6)
        time (str date "T00:00:00")
        task {:id (gen/uuid rng)
              :query (gen/cyrillic rng 6)
              :status (gen/greek rng 6)
              :language (gen/cyrillic rng 5)
              :service tag
              :created time
              :result {:summary note
                       :sources []}}
        hold {:run_id (gen/uuid rng)
              :query (gen/cyrillic rng 6)
              :processor (gen/greek rng 6)
              :language (gen/cyrillic rng 6)
              :provider tag}
        data {:id (gen/uuid rng)
              :topic (gen/cyrillic rng 6)
              :tasks [task]
              :pending hold
              :created time}
        file (.resolve path "session.edn")
        _ (spit (.toFile file) (pr-str data) :encoding "UTF-8")
        repo (repo/repo root)
        _ (repo/load repo)
        body (slurp (.toFile file) :encoding "UTF-8")
        flag (and (not (re-find #":query" body))
                  (not (re-find #":result" body))
                  (re-find #":brief" body))]
    (is flag "Session edn still included query or result or missed brief")))
