(ns research.main.execute
  (:require [clojure.string :as str]
            [research.api.parallel :as parallel]
            [research.api.research :as research]
            [research.api.response :as response]
            [research.api.valyu :as valyu]
            [research.api.xai :as xai]
            [research.domain.pending :as pending]
            [research.domain.result :as result]
            [research.domain.session :as session]
            [research.domain.task :as task]
            [research.image.frame :as frame]
            [research.image.generator :as image]
            [research.main.support :as support]
            [research.pdf.document :as document]
            [research.pdf.palette :as palette]
            [research.storage.organizer :as organizer]
            [research.storage.repository :as repo])
  (:import (java.util Optional UUID)))

(defn execute
  "Run research for session."
  [root data out id env]
  (let [repo (repo/repo data)
        list (repo/load repo)
        pick (first (filter #(str/starts-with? (session/id %) id) list))]
    (if (not pick)
      (println (str "Session not found: " id))
      (let [pending (session/pending pick)]
        (println (str "Session: " (session/topic pick)))
        (if (.isPresent pending)
          (let [pend (.get pending)
                run (pending/id pend)
                query (pending/query pend)
                processor (pending/processor pend)
                language (pending/language pend)
                provider (pending/provider pend)
                exec (cond
                       (= provider "valyu")
                       (valyu/valyu {:key (env "VALYU_API_KEY")})
                       (= provider "xai")
                       (xai/xai {:root root
                                 :mode (if (= processor "full")
                                         "full"
                                         "social_multi")})
                       :else (parallel/parallel))]
            (println (str "Resuming run: "
                          (subs run 0 (min 16 (count run)))))
            (println (str "Query: " query))
            (println (str "Processor: " processor))
            (println "Streaming progress")
            (research/stream exec run)
            (println "Fetching result")
            (let [resp (research/finish exec run)
                  updated (session/reset pick)
                  _ (repo/update repo updated)
                  org (organizer/organizer out)
                  name (organizer/name
                        org
                        (session/created pick)
                        (session/topic pick)
                        (session/id pick))
                  _ (organizer/response org name provider (response/raw resp))
                  _ (support/store name provider (response/raw resp) out)
                  summary (response/text resp)
                  sources (response/sources resp)
                  pack {:summary summary
                        :sources (mapv result/data sources)}
                  brief (pending/brief pend)
                  task (task/task {:id (str (UUID/randomUUID))
                                   :query query
                                   :status "completed"
                                   :language language
                                   :service (if (= provider "xai")
                                              "x.ai"
                                              (str provider ".ai"))
                                   :processor processor
                                   :brief brief
                                   :created (task/format (task/now))
                                   :result pack})
                  final (session/extend updated task)
                  _ (repo/update repo final)
                  cover (organizer/cover org name provider)
                  coveropt (Optional/of cover)
                  key (or (env "GEMINI_API_KEY") "")]
              (let [file (organizer/folder org name provider)
                    count (count (response/sources resp))]
                (println (str "Response saved: " (.toString file)))
                (println (str "Results saved: " count " sources")))
              (if (str/blank? key)
                (println "Gemini API key not set skipping image generation")
                (do (println "Generating cover image")
                    (let [gen (image/generator)
                          detector (frame/detector)]
                      (try
                        (frame/retry gen detector (session/topic final) cover 4)
                        (println (str "Cover generated: "
                                      (.toString cover)))
                        (catch Exception cause
                          (let [data (ex-data cause)
                                status (or (:status data) "none")
                                model (or (:model data) "unknown")]
                            (println
                             (str "Cover generation failed model="
                                  model
                                  " status="
                                  status))))))))
              (let [doc (document/document
                         final
                         (palette/palette)
                         coveropt
                         out)
                    path (organizer/report org name provider)]
                (document/save doc path)
                (println (str "PDF generated: " (.toString path))))))
          (let [query (session/query pick)
                processor (session/processor pick)
                language (session/language pick)
                provider (session/provider pick)
                allow #{"parallel" "valyu" "xai"}
                _ (when-not (contains? allow provider)
                    (throw (ex-info
                            "Provider must be parallel valyu or xai"
                            {})))
                check (or (= processor "fast")
                          (= processor "standard")
                          (= processor "heavy"))
                _ (when (and (= provider "valyu") (not check))
                    (throw
                     (ex-info
                      "Processor must be fast standard or heavy for valyu"
                      {})))
                exec (cond
                       (= provider "valyu")
                       (valyu/valyu {:key (env "VALYU_API_KEY")})
                       (= provider "xai")
                       (xai/xai {:root root
                                 :mode (if (= processor "full")
                                         "full"
                                         "social_multi")})
                       :else (parallel/parallel))
                run (research/start exec query processor)
                pend (pending/pending {:run_id run
                                       :query query
                                       :processor processor
                                       :language language
                                       :provider provider
                                       :topic (session/topic pick)})
                state (session/start pick pend)
                _ (repo/update repo state)]
            (println (str "Query: " query))
            (println (str "Processor: " processor))
            (println (str "Language: " language))
            (println (str "Research started: " run))
            (println "Streaming progress")
            (research/stream exec run)
            (println "Fetching result")
            (let [resp (research/finish exec run)
                  updated (session/reset pick)
                  _ (repo/update repo updated)
                  org (organizer/organizer out)
                  name (organizer/name
                        org
                        (session/created pick)
                        (session/topic pick)
                        (session/id pick))
                  _ (organizer/response org name provider (response/raw resp))
                  _ (support/store name provider (response/raw resp) out)
                  summary (response/text resp)
                  sources (response/sources resp)
                  pack {:summary summary
                        :sources (mapv result/data sources)}
                  brief (pending/brief pend)
                  task (task/task {:id (str (UUID/randomUUID))
                                   :query query
                                   :status "completed"
                                   :language language
                                   :service (if (= provider "xai")
                                              "x.ai"
                                              (str provider ".ai"))
                                   :processor processor
                                   :brief brief
                                   :created (task/format (task/now))
                                   :result pack})
                  final (session/extend updated task)
                  _ (repo/update repo final)
                  cover (organizer/cover org name provider)
                  coveropt (Optional/of cover)
                  key (or (env "GEMINI_API_KEY") "")]
              (let [file (organizer/folder org name provider)
                    count (count (response/sources resp))]
                (println (str "Response saved: " (.toString file)))
                (println (str "Results saved: " count " sources")))
              (if (str/blank? key)
                (println "Gemini API key not set skipping image generation")
                (do (println "Generating cover image")
                    (let [gen (image/generator)
                          detector (frame/detector)]
                      (try
                        (frame/retry gen detector (session/topic final) cover 4)
                        (println (str "Cover generated: "
                                      (.toString cover)))
                        (catch Exception cause
                          (let [data (ex-data cause)
                                status (or (:status data) "none")
                                model (or (:model data) "unknown")]
                            (println
                             (str "Cover generation failed model="
                                  model
                                  " status="
                                  status))))))))
              (let [doc (document/document
                         final
                         (palette/palette)
                         coveropt
                         out)
                    path (organizer/report org name provider)]
                (document/save doc path)
                (println (str "PDF generated: " (.toString path)))))))))))
