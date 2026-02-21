(ns research.main.launch
  (:require [research.main.execute :as execute]
            [research.main.seed :as seed]))

(defn launch
  "Create session and run research."
  [root data topic query processor language provider]
  (let [processor (if (and (= provider "xai") (= processor "year"))
                    "social"
                    processor)
        id (seed/seed data topic)
        mode (cond
               (= processor "lite")
               (throw (ex-info
                       "Run failed because processor lite is not supported"
                       {:processor processor}))
               (and (= provider "xai")
                    (not (or (= processor "social")
                             (= processor "full"))))
               (throw (ex-info
                       (str "Run failed because processor"
                            " must be social or full for xai")
                       {:processor processor}))
               (and (= provider "valyu")
                    (not (or (= processor "fast")
                             (= processor "standard")
                             (= processor "heavy"))))
               (throw (ex-info
                       (str "Run failed because processor is not supported"
                            " for valyu")
                       {:processor processor}))
               (or (= processor "fast")
                   (= processor "standard")
                   (= processor "heavy"))
               processor
               :else "standard")
        pairs (if (= provider "all")
                [["parallel" processor]
                 ["valyu" mode]]
                [[provider
                  (if (= provider "valyu") mode processor)]])]
    (doseq [pair pairs]
      (let [name (first pair)
            proc (second pair)]
        (execute/execute root data id query proc language name)))))
