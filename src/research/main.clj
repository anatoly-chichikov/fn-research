(ns research.main
  (:refer-clojure :exclude [list])
  (:require [clojure.java.io :as io]
            [clojure.string :as str]
            [clojure.tools.cli :as cli]
            [research.domain.session :as session]
            [research.main.execute :as execute]
            [research.main.launch :as launch]
            [research.main.print :as print]
            [research.main.seed :as seed]
            [research.main.support :as support]
            [research.storage.repository :as repo]))

(declare env)

(defprotocol Applied
  "Object that runs CLI operations."
  (list [item] "List sessions.")
  (show [item id] "Show session details.")
  (generate [item id html] "Generate report for session.")
  (create [item topic] "Create session.")
  (run [item topic query processor language provider]
    "Create session and run research.")
  (research [item id query processor language provider]
    "Run research for session."))

(defrecord App [root data out]
  Applied
  (list [_]
    (print/enumerate data))
  (show [_ id]
    (print/display data id))
  (generate [_ id html]
    (print/render data out id html))
  (create [_ topic]
    (seed/seed data topic "" "" "" ""))
  (run [_ topic query processor language provider]
    (launch/launch root data out topic query processor language provider env))
  (research [_ id query processor language provider]
    (let [store (repo/repo data)
          list (repo/load store)
          pick (first (filter #(str/starts-with? (session/id %) id) list))]
      (when pick
        (let [updated (session/reconfigure
                       pick
                       {:query query
                        :processor processor
                        :language language
                        :provider provider})]
          (repo/update store updated)
          (execute/execute root data out id env))))))

(def env
  "Return environment value by key."
  support/env)

(defn app
  "Create application instance."
  [root]
  (let [out (.resolve root "output")
        data out]
    (->App root data out)))

(defn parse
  "Parse CLI arguments."
  [args]
  (let [opts [[nil "--processor PROCESSOR" "Processor" :default "pro"]
              [nil "--language LANGUAGE" "Language" :default "русский"]
              [nil "--provider PROVIDER" "Provider" :default "parallel"]
              [nil "--html" "HTML" :default false]]
        parse (cli/parse-opts args opts)
        list (:arguments parse)
        opts (:options parse)
        cmd (first list)
        tail (rest list)]
    {:cmd cmd
     :tail tail
     :opts opts}))

(defn -main
  "Entry point."
  [& args]
  (let [root (.toPath (io/file "."))
        app (app root)
        data (parse args)
        cmd (:cmd data)
        tail (:tail data)
        opts (:opts data)]
    (case cmd
      "list" (list app)
      "show" (show app (first tail))
      "generate" (generate app (first tail) (:html opts))
      "create" (create app (str/join " " tail))
      "run" (run app
                 (first tail)
                 (second tail)
                 (:processor opts)
                 (:language opts)
                 (:provider opts))
      "research" (research app
                           (first tail)
                           (second tail)
                           (:processor opts)
                           (:language opts)
                           (:provider opts))
      (println "Unknown command"))
    (shutdown-agents)))
