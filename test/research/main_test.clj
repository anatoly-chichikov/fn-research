(ns research.main-test
  (:require [clojure.edn :as edn]
            [clojure.test :refer [deftest is]]
            [jsonista.core :as json]
            [research.api.parallel :as parallel]
            [research.api.research :as research]
            [research.api.response :as response]
            [research.api.valyu :as valyu]
            [research.domain.pending :as pending]
            [research.domain.session :as session]
            [research.image.generator :as image]
            [research.main :as main]
            [research.main.support :as support]
            [research.pdf.document :as document]
            [research.storage.organizer :as organizer]
            [research.storage.repository :as repo]
            [research.test.ids :as gen])
  (:import (java.io StringWriter)
           (java.nio.file Files Paths)
           (java.nio.file.attribute FileAttribute)
           (javax.imageio ImageIO)
           (org.apache.pdfbox.pdmodel PDDocument)
           (org.apache.pdfbox.rendering PDFRenderer)))

(defn screens
  "Render PDF pages into images."
  [path folder dpi]
  (with-open [doc (PDDocument/load (.toFile path))]
    (let [rend (PDFRenderer. doc)
          size (.getNumberOfPages doc)]
      (loop [idx 0 list []]
        (if (< idx size)
          (let [image (.renderImageWithDPI rend idx dpi)
                name (format "page-%03d.png" (inc idx))
                file (.resolve folder name)
                _ (ImageIO/write image "png" (.toFile file))]
            (recur (inc idx) (conj list image)))
          list)))))

(defn mismatch
  "Return first mismatching page index."
  [left right]
  (let [size (count left)
        total (count right)
        head (min size total)]
    (if (not= size total)
      (inc head)
      (loop [idx 0]
        (if (< idx size)
          (let [one (nth left idx)
                two (nth right idx)
                w1 (.getWidth one)
                h1 (.getHeight one)
                w2 (.getWidth two)
                h2 (.getHeight two)]
            (if (or (not= w1 w2) (not= h1 h2))
              (inc idx)
              (let [a1 (.getRGB one 0 0 w1 h1 nil 0 w1)
                    a2 (.getRGB two 0 0 w2 h2 nil 0 w2)]
                (if (java.util.Arrays/equals a1 a2)
                  (recur (inc idx))
                  (inc idx)))))
          0)))))

(deftest the-cli-parses-command
  (let [rng (gen/ids 25001)
        text (gen/cyrillic rng 6)
        data (main/parse ["create" text])]
    (is (= "create" (:cmd data)) "CLI parse is incorrect")))

(deftest the-cli-parses-options
  (let [rng (gen/ids 25002)
        topic (gen/cyrillic rng 6)
        query (gen/cyrillic rng 7)
        processor (gen/cyrillic rng 5)
        language (gen/cyrillic rng 4)
        provider (gen/cyrillic rng 6)
        data (main/parse ["run"
                          topic
                          query
                          "--processor"
                          processor
                          "--language"
                          language
                          "--provider"
                          provider])
        value (:opts data)
        goal {:processor processor
              :language language
              :provider provider
              :html false}]
    (is (= goal value) "Options were not parsed")))

(deftest the-application-run-forwards-parameters
  (let [rng (gen/ids 25003)
        topic (gen/cyrillic rng 6)
        query (gen/greek rng 7)
        code (gen/armenian rng 5)
        processor (gen/cyrillic rng 4)
        language (gen/greek rng 4)
        provider (gen/hiragana rng 4)
        store (atom nil)
        item (reify main/Applied
               (list [_] nil)
               (show [_ _] nil)
               (generate [_ _ _] nil)
               (create [_ value] (reset! store [value]) code)
               (run [self value query processor language provider]
                 (let [code (main/create self value)]
                   (main/research self code query processor language provider)))
               (research [_ code query processor language provider]
                 (reset! store [(first @store)
                                code
                                query
                                processor
                                language
                                provider])))]
    (main/run item topic query processor language provider)
    (is (= [topic code query processor language provider] @store)
        "Run did not pass data")))

(deftest the-application-run-executes-all-providers
  (let [rng (gen/ids 25004)
        topic (gen/cyrillic rng 6)
        query (gen/greek rng 7)
        processor (gen/cyrillic rng 4)
        language (gen/greek rng 4)
        root (Files/createTempDirectory "app"
                                        (make-array FileAttribute 0))
        app (main/app root)
        runs (atom [])
        alpha (reify research/Researchable
                (start [_ query processor]
                  (swap! runs conj ["alpha" query processor])
                  "alpha-run")
                (stream [_ _] true)
                (finish [_ id]
                  (response/response {:id id
                                      :status "completed"
                                      :output (gen/cyrillic rng 6)
                                      :basis []
                                      :raw {}})))
        beta (reify research/Researchable
               (start [_ query processor]
                 (swap! runs conj ["beta" query processor])
                 "beta-run")
               (stream [_ _] true)
               (finish [_ id]
                 (response/response {:id id
                                     :status "completed"
                                     :output (gen/cyrillic rng 6)
                                     :basis []
                                     :raw {}})))]
    (with-redefs [parallel/parallel (fn [] alpha)
                  valyu/valyu (fn [_] beta)
                  support/env (fn [_] "")
                  document/emit (fn [_ _] nil)
                  image/generate (fn [_ _ _] nil)]
      (main/run app topic query processor language "all"))
    (let [total (count @runs)
          uniques (count (distinct (map first @runs)))]
      (is (and (= 2 total) (= 2 uniques))
          "Run did not execute two providers for all"))))

(deftest ^{:doc "Ensure valyu rejects legacy processor."}
  the-application-run-rejects-valyu-lite-processor
  (let [rng (gen/ids 25006)
        topic (gen/cyrillic rng 6)
        query (gen/greek rng 7)
        processor "lite"
        language (gen/greek rng 4)
        root (Files/createTempDirectory "app"
                                        (make-array FileAttribute 0))
        app (main/app root)]
    (is (thrown? clojure.lang.ExceptionInfo
                 (main/run app topic query processor language "valyu"))
        "Valyu accepted legacy processor")))

(deftest the-application-skips-cover-when-key-missing
  (let [rng (gen/ids 25005)
        topic (gen/cyrillic rng 6)
        query (gen/greek rng 7)
        processor (gen/armenian rng 5)
        language (gen/hiragana rng 4)
        provider "parallel"
        run (gen/arabic rng 8)
        text (gen/cyrillic rng 12)
        stamp (session/format (session/now))
        ident (gen/uuid rng)
        entry {:run_id run
               :query query
               :processor processor
               :language language
               :provider provider}
        sess (session/session {:id ident
                               :topic topic
                               :tasks []
                               :created stamp
                               :pending entry})
        root (Files/createTempDirectory "app"
                                        (make-array FileAttribute 0))
        out (.resolve root "output")
        _ (Files/createDirectories out (make-array FileAttribute 0))
        store (repo/repo out)
        _ (repo/save store [sess])
        reply (response/response {:id run
                                  :status "completed"
                                  :output text
                                  :basis []
                                  :raw {}})
        fake (reify research/Researchable
               (start [_ _ _] run)
               (stream [_ _] true)
               (finish [_ _] reply))
        app (main/app root)
        token (subs ident 0 8)]
    (with-redefs [parallel/parallel (fn [] fake)
                  support/env (fn [_] "")
                  document/emit (fn [_ _] nil)
                  image/generate (fn [_ _ _] nil)]
      (main/research app token query processor language provider))
    (let [org (organizer/organizer out)
          name (organizer/name
                org
                (session/created sess)
                (session/topic sess)
                (session/id sess))
          cover (organizer/cover org name provider)]
      (is (not (Files/exists cover (make-array java.nio.file.LinkOption 0)))
          "Cover image was generated despite missing key"))))

(deftest the-application-writes-raw-response
  (let [rng (gen/ids 25007)
        topic (gen/cyrillic rng 6)
        query (gen/greek rng 7)
        processor (gen/armenian rng 5)
        language (gen/hiragana rng 4)
        provider (gen/cyrillic rng 5)
        run (gen/arabic rng 8)
        text (gen/cyrillic rng 12)
        stamp (session/format (session/now))
        ident (gen/uuid rng)
        entry {:run_id run
               :query query
               :processor processor
               :language language
               :provider provider}
        sess (session/session {:id ident
                               :topic topic
                               :tasks []
                               :created stamp
                               :pending entry})
        root (Files/createTempDirectory "app"
                                        (make-array FileAttribute 0))
        out (.resolve root "output")
        _ (Files/createDirectories out (make-array FileAttribute 0))
        store (repo/repo out)
        _ (repo/save store [sess])
        key (keyword (gen/cyrillic rng 6))
        value (gen/greek rng 6)
        nest (keyword (gen/armenian rng 5))
        inner (keyword (gen/hiragana rng 4))
        raw {key value
             nest {inner (.nextInt rng 1000)}}
        reply (response/response {:id run
                                  :status "completed"
                                  :output text
                                  :basis []
                                  :raw raw})
        fake (reify research/Researchable
               (start [_ _ _] run)
               (stream [_ _] true)
               (finish [_ _] reply))
        app (main/app root)
        token (subs ident 0 8)]
    (with-redefs [parallel/parallel (fn [] fake)
                  support/env (fn [_] "")
                  document/emit (fn [_ _] nil)
                  image/generate (fn [_ _ _] nil)]
      (main/research app token query processor language provider))
    (let [org (organizer/organizer out)
          name (organizer/name
                org
                (session/created sess)
                (session/topic sess)
                (session/id sess))
          tag (organizer/slug provider)
          tag (if (empty? tag) "provider" tag)
          folder (organizer/folder org name provider)
          path (.resolve folder (str "response-" tag ".json"))
          data (json/read-value
                (.toFile path)
                (json/object-mapper {:decode-key-fn keyword}))]
      (is (= raw data) "Raw response did not match stored response"))))

(deftest the-application-continues-after-cover-failure
  (let [rng (gen/ids 25008)
        topic (gen/cyrillic rng 6)
        query (gen/greek rng 7)
        processor (gen/armenian rng 5)
        language (gen/hiragana rng 4)
        provider (gen/cyrillic rng 5)
        run (gen/arabic rng 8)
        text (gen/cyrillic rng 12)
        stamp (session/format (session/now))
        ident (gen/uuid rng)
        entry {:run_id run
               :query query
               :processor processor
               :language language
               :provider provider}
        sess (session/session {:id ident
                               :topic topic
                               :tasks []
                               :created stamp
                               :pending entry})
        root (Files/createTempDirectory "app"
                                        (make-array FileAttribute 0))
        out (.resolve root "output")
        _ (Files/createDirectories out (make-array FileAttribute 0))
        store (repo/repo out)
        _ (repo/save store [sess])
        reply (response/response {:id run
                                  :status "completed"
                                  :output text
                                  :basis []
                                  :raw {}})
        fake (reify research/Researchable
               (start [_ _ _] run)
               (stream [_ _] true)
               (finish [_ _] reply))
        app (main/app root)
        token (subs ident 0 8)
        org (organizer/organizer out)
        name (organizer/name
              org
              (session/created sess)
              (session/topic sess)
              (session/id sess))
        path (organizer/report org name provider)
        boom (ex-info "Cover generation failed model=none status=none" {})]
    (with-redefs [parallel/parallel (fn [] fake)
                  support/env (fn [key]
                                (if (= key "GEMINI_API_KEY")
                                  (gen/latin rng 6)
                                  ""))
                  image/generator (fn [] nil)
                  image/generate (fn [_ _ _] (throw boom))]
      (main/research app token query processor language provider))
    (is (Files/exists path (make-array java.nio.file.LinkOption 0))
        "Report was not generated after cover failure")))

(deftest the-application-saves-brief-in-session
  (let [rng (gen/ids 25009)
        topic (gen/cyrillic rng 6)
        query (str (gen/cyrillic rng 5) "\n\n" (gen/greek rng 7))
        processor "pro"
        language (gen/cyrillic rng 4)
        provider "parallel"
        run (gen/arabic rng 8)
        text (gen/cyrillic rng 12)
        stamp (session/format (session/now))
        ident (gen/uuid rng)
        sess (session/session {:id ident
                               :topic topic
                               :tasks []
                               :created stamp})
        root (Files/createTempDirectory "app"
                                        (make-array FileAttribute 0))
        out (.resolve root "output")
        _ (Files/createDirectories out (make-array FileAttribute 0))
        store (repo/repo out)
        _ (repo/save store [sess])
        reply (response/response {:id run
                                  :status "completed"
                                  :output text
                                  :basis []
                                  :raw {}})
        fake (reify research/Researchable
               (start [_ _ _] run)
               (stream [_ _] true)
               (finish [_ _] reply))
        app (main/app root)
        token (subs ident 0 8)]
    (with-redefs [parallel/parallel (fn [] fake)
                  support/env (fn [_] "")
                  document/emit (fn [_ _] nil)
                  image/generate (fn [_ _ _] nil)]
      (main/research app token query processor language provider))
    (let [org (organizer/organizer out)
          name (organizer/name
                org
                (session/created sess)
                (session/topic sess)
                (session/id sess))
          folder (organizer/folder org name provider)
          path (.resolve folder "session.edn")
          data (edn/read-string (slurp (.toFile path) :encoding "UTF-8"))
          brief (get-in data [:tasks 0 :brief])
          seen (and (contains? brief :topic)
                    (contains? brief :items)
                    (not (contains? brief :text)))]
      (is seen "Brief was not stored in session edn"))))

(deftest ^{:doc "Ensure pending brief structure is preserved after run."}
  the-application-preserves-brief-structure
  (let [rng (gen/ids 25012)
        topic (gen/cyrillic rng 6)
        text (gen/greek rng 6)
        leaf (gen/hiragana rng 6)
        node (gen/armenian rng 6)
        processor "pro"
        language (gen/cyrillic rng 4)
        provider "parallel"
        run (gen/arabic rng 8)
        stamp (session/format (session/now))
        ident (gen/uuid rng)
        brief {:topic topic
               :items [{:text text
                        :items [{:text leaf
                                 :items []}
                                {:text node
                                 :items []}]}]}
        entry {:run_id run
               :brief brief
               :processor processor
               :language language
               :provider provider}
        pend (pending/pending entry)
        query (pending/query pend)
        sess (session/session {:id ident
                               :topic topic
                               :tasks []
                               :created stamp
                               :pending entry})
        root (Files/createTempDirectory "app"
                                        (make-array FileAttribute 0))
        out (.resolve root "output")
        _ (Files/createDirectories out (make-array FileAttribute 0))
        store (repo/repo out)
        _ (repo/save store [sess])
        reply (response/response {:id run
                                  :status "completed"
                                  :output (gen/cyrillic rng 6)
                                  :basis []
                                  :raw {}})
        fake (reify research/Researchable
               (start [_ _ _] run)
               (stream [_ _] true)
               (finish [_ _] reply))
        app (main/app root)
        token (subs ident 0 8)]
    (with-redefs [parallel/parallel (fn [] fake)
                  support/env (fn [_] "")
                  document/emit (fn [_ _] nil)
                  image/generate (fn [_ _ _] nil)]
      (main/research app token query processor language provider))
    (let [org (organizer/organizer out)
          name (organizer/name
                org
                (session/created sess)
                (session/topic sess)
                (session/id sess))
          folder (organizer/folder org name provider)
          path (.resolve folder "session.edn")
          data (edn/read-string (slurp (.toFile path) :encoding "UTF-8"))
          item (get-in data [:tasks 0 :brief])
          node (first (:items item))
          seen (seq (or (:items node) []))]
      (is seen "Nested brief items were not preserved"))))

(deftest ^:integration the-application-generates-pdf-screenshots
  (let [rng (gen/ids 25011)
        lang (gen/cyrillic rng 6)
        head (gen/ascii rng 6)
        base (Paths/get "baseline-research" (make-array String 0))
        brief (edn/read-string
               (slurp (.toFile (.resolve base "brief-parallel.edn"))
                      :encoding "UTF-8"))
        raw (json/read-value
             (.toFile (.resolve base "response-parallel.json"))
             (json/object-mapper {:decode-key-fn keyword}))
        cover (.resolve base "cover-parallel.jpg")
        gold (.resolve base "baseline.pdf")
        author "Anatoly Chichikov"
        root (Files/createTempDirectory head (make-array FileAttribute 0))
        out (.resolve root "output")
        _ (Files/createDirectories out (make-array FileAttribute 0))
        repo (repo/repo out)
        stamp "2026-01-01T00:00:00"
        ident (gen/uuid rng)
        sess (session/session {:id ident
                               :topic "Clojure production pain points"
                               :tasks []
                               :created stamp})
        _ (repo/save repo [sess])
        run (gen/ascii rng 8)
        query (pending/query
               (pending/pending {:run_id run
                                 :brief brief
                                 :processor "pro"
                                 :language lang
                                 :provider "parallel"}))
        fake (reify research/Researchable
               (start [_ _ _] run)
               (stream [_ _] true)
               (finish [_ _]
                 (let [output (:output raw)
                       text (if (map? output) (or (:content output) "") "")
                       basis (if (map? output) (or (:basis output) []) [])
                       info (or (:run raw) {})
                       code (or (:run_id info) run)
                       state (or (:status info) "completed")]
                   (response/response {:id code
                                       :status state
                                       :output text
                                       :basis basis
                                       :raw raw}))))
        app (main/app root)
        cache (Paths/get "tmp_cache" (make-array String 0))
        folder (.resolve cache (str "pdf-screens-" head))
        left (.resolve folder "baseline")
        right (.resolve folder "generated")
        org (organizer/organizer out)
        label (organizer/name
               org
               (session/created sess)
               (session/topic sess)
               (session/id sess))
        path (organizer/report org label "parallel")]
    (with-redefs [parallel/parallel (fn [] fake)
                  support/env (fn [key] (if (= key "GEMINI_API_KEY") "key" ""))
                  document/env (fn [_] author)
                  image/generator (fn [] nil)
                  image/generate (fn [_ _ target]
                                   (Files/copy
                                    cover
                                    target
                                    (make-array java.nio.file.CopyOption 0)))]
      (binding [*out* (StringWriter.) *err* (StringWriter.)]
        (main/research app (subs ident 0 8) query "pro" lang "parallel")))
    (Files/createDirectories left (make-array FileAttribute 0))
    (Files/createDirectories right (make-array FileAttribute 0))
    (let [lefts (screens gold left 150)
          rights (screens path right 150)
          miss (mismatch lefts rights)
          text (if (zero? miss)
                 "Screenshot mismatch detected"
                 (let [name (format "page-%03d.png" miss)
                       base (.toString
                             (.toUri
                              (.resolve left name)))
                       gen (.toString
                            (.toUri
                             (.resolve right name)))]
                   (str "Page "
                        miss
                        " screenshot did not match baseline "
                        "baseline-url="
                        base
                        " generated-url="
                        gen)))]
      (is (zero? miss) text))))
