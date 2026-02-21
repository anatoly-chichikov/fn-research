(ns research.pdf.document.env
  (:require [clojure.java.shell :as shell]
            [clojure.string :as str]
            [markdown.core :as md]
            [research.domain.pending :as pending]
            [research.domain.session :as sess]
            [research.domain.task :as task]
            [research.pdf.document.citations :as cite]
            [research.pdf.document.text :as text])
  (:import (java.nio.file Files LinkOption)
           (java.nio.file.attribute FileAttribute)))

(defn env
  "Return environment value by key."
  [key]
  (System/getenv key))

(defn emit
  "Render PDF using WeasyPrint."
  [html path]
  (let [tmp (Files/createTempFile
             "report" ".html"
             (make-array FileAttribute 0))]
    (try
      (let [_ (spit (.toFile tmp) html :encoding "UTF-8")
            vars (into {} (System/getenv))
            home (or (get vars "DYLD_FALLBACK_LIBRARY_PATH") "")
            list [home "/opt/homebrew/lib" "/usr/local/lib"]
            list (filter #(not (str/blank? %)) list)
            link (str/join ":" list)
            vars (assoc vars "DYLD_FALLBACK_LIBRARY_PATH" link)
            res (shell/sh
                 "uv"
                 "run"
                 "--with"
                 "weasyprint"
                 "python"
                 "-m"
                 "weasyprint"
                 (.toString tmp)
                 (.toString path)
                 :env vars)
            code (:exit res)]
        (if (zero? code)
          path
          (throw (ex-info "Weasyprint failed" {:code code
                                               :out (:out res)
                                               :err (:err res)}))))
      (finally
        (Files/deleteIfExists tmp)))))

(defn author
  "Return report author from env."
  []
  (or (env "REPORT_FOR") ""))

(defn service
  "Return service name from latest task."
  [item]
  (let [list (sess/tasks item)
        last (last list)]
    (if last (task/provider last) "parallel.ai")))

(defn coverimage
  "Render cover image html."
  [item]
  (let [cover (:cover item)]
    (if (and (.isPresent cover)
             (Files/exists (.get cover) (make-array LinkOption 0)))
      (str "<div class=\"cover-image\"><img src=\""
           (.toString (.toUri (.get cover)))
           "\" alt=\"Cover\" /></div>")
      "")))

(defn brief
  "Render brief section."
  [item]
  (let [sess (:session item)
        list (sess/tasks sess)
        head (first list)
        hold (sess/pending sess)
        slot (if (and (not head) (.isPresent hold)) (.get hold) nil)
        info (cond
               head (task/brief head)
               slot (pending/brief slot)
               :else {})
        items (or (:items info) [])
        topic (str (or (:topic info) ""))
        text (cond
               (seq items) ""
               (str/blank? topic) ""
               :else topic)
        text (if (seq items) "" (text/listify text))
        text (if (seq items) "" (text/normalize text))
        text (if (seq items) "" (text/rule text))
        text (if (seq items) "" (cite/stars text))
        html (if (seq items)
               (str
                (if (str/blank? topic)
                  ""
                  (str "<p>" (text/escape topic) "</p>"))
                (text/outline items))
               (md/md-to-html-string text))
        html (if (seq items) html (cite/tables html))
        html (if (seq items) html (cite/codeindent html))
        html (if (seq items) html (cite/backslash html))]
    (if (str/blank? html)
      ""
      (str "<div class=\"brief\"><div class=\"container\">"
           "<h1>Exploration Brief</h1>"
           "<div class=\"query\">"
           html
           "</div></div></div>"))))

(defn provider
  "Return provider slug from task service."
  [item]
  (let [name (task/provider item)]
    (cond
      (= name "x.ai") "xai"
      (str/ends-with? name ".ai") (first (str/split name #"\."))
      :else name)))
