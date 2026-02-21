(ns research.main.launch
  (:require [clojure.string :as str]
            [research.domain.session :as session]
            [research.main.execute :as execute]
            [research.main.seed :as seed]
            [research.storage.repository :as repo]))

(defn launch
  "Create session and run research."
  [root data out topic query processor language provider env]
  (let [processor (if (and (= provider "xai") (= processor "year"))
                    "social"
                    processor)
        _ (when (= processor "lite")
            (throw (ex-info
                    "Run failed because processor lite is not supported"
                    {:processor processor})))
        _ (when (and (= provider "xai")
                     (not (or (= processor "social")
                              (= processor "full"))))
            (throw (ex-info
                    (str "Run failed because processor"
                         " must be social or full for xai")
                    {:processor processor})))
        _ (when (and (= provider "valyu")
                     (not (or (= processor "fast")
                              (= processor "standard")
                              (= processor "heavy"))))
            (throw (ex-info
                    (str "Run failed because processor is not supported"
                         " for valyu")
                    {:processor processor})))
        mode (cond
               (or (= processor "fast")
                   (= processor "standard")
                   (= processor "heavy"))
               processor
               :else "standard")
        pairs (if (= provider "all")
                [["parallel" processor]
                 ["valyu" mode]]
                [[provider
                  (if (= provider "valyu") mode processor)]])
        first-pair (first pairs)
        id (seed/seed data topic query (second first-pair) language
                      (first first-pair))]
    (execute/execute root data out id env)
    (when (> (count pairs) 1)
      (let [pair (second pairs)
            store (repo/repo data)
            list (repo/load store)
            pick (first (filter #(str/starts-with?
                                  (session/id %) id) list))
            updated (session/reconfigure
                     pick
                     {:provider (first pair)
                      :processor (second pair)})]
        (repo/update store updated)
        (execute/execute root data out id env)))))
