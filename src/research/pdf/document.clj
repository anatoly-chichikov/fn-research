(ns research.pdf.document
  (:require [clojure.string :as str]
            [markdown.core :as md]
            [research.domain.pending :as pending]
            [research.domain.session :as sess]
            [research.domain.task :as task]
            [research.pdf.document.citations :as doccite]
            [research.pdf.document.data :as docdata]
            [research.pdf.document.env :as docenv]
            [research.pdf.document.sources :as docsources]
            [research.pdf.document.tasks :as doctasks]
            [research.pdf.document.text :as doctext]
            [research.pdf.style :as style])
  (:import (java.nio.file Files LinkOption)))
(declare author service coverimage brief emit title)
(defprotocol Signed
  "Object with author signature."
  (html [item] "Return HTML signature."))
(defprotocol Rendered
  "Object that can render document."
  (render [item] "Return HTML document."))
(defprotocol Exported
  "Object that can export to file."
  (save [item path] "Save PDF to path.")
  (page [item path] "Save HTML to path."))
(defrecord Signature [name service]
  Signed
  (html [_]
    (let [repo "https://github.com/anatoly-chichikov/defn-research"
          site (str "https://" service)
          link (str "<a href=\"" repo "\">defn research</a>")
          host (str "<a href=\"" site "\">" service "</a>")
          mark (str "<span class=\"signature-mark\">("
                    link
                    " ["
                    host
                    "]&hairsp;)</span>")
          text (if (str/blank? name)
                 (str "AI generated report with " mark)
                 (str "AI generated report for <span class=\"author\">"
                      name
                      "</span> with "
                      mark))]
      (str text "<br>May contain inaccuracies, please verify"))))
(defrecord Document [session palette cover root]
  Rendered
  (render [item]
    (let [[content _] (doctasks/tasks item)
          list (docsources/catalog item)
          extra (docsources/section list)
          sign (->Signature (author) (service session))
          note (html sign)
          css (style/css (style/style palette))
          form java.time.format.DateTimeFormatter/ISO_LOCAL_DATE
          stamp (.format (sess/created session) form)
          brief (brief item)
          body (str brief
                    "<div class=\"container content\">"
                    "<div class=\"tasks\">"
                    content
                    "</div>"
                    "</div><div class=\"container\">"
                    extra
                    "</div>")
          data (doctext/anchors body)
          toc (doctext/toc (:items data))
          body (:html data)]
      (str "<!DOCTYPE html><html lang=\"en\"><head>"
           "<meta charset=\"UTF-8\" />"
           "<title>"
           (doctext/escape (doctext/heading (title session)))
           "</title><style>"
           css
           "</style></head><body>"
           "<div class=\"page-footer\">"
           note
           "</div>"
           "<div class=\"intro\">"
           (coverimage item)
           "<div class=\"intro-content\"><h1>"
           (doctext/escape (doctext/heading (title session)))
           "</h1><div class=\"meta\"><p class=\"subtitle\">"
           note
           "</p><p class=\"date\">"
           stamp
           "</p></div></div></div>"
           toc
           body
           "</body></html>")))
  Exported
  (save [item path]
    (let [html (render item)]
      (emit html path)))
  (page [item path]
    (let [html (render item)
          _ (spit (.toFile path) html :encoding "UTF-8")]
      path))
  Object
  (toString [item] (render item)))
(def env "Env value." docenv/env)
(def emit "Render PDF." docenv/emit)
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
(defn title
  "Return research title from session topic or brief."
  [session]
  (let [topic (sess/topic session)]
    (if (str/blank? topic)
      (let [list (sess/tasks session)
            head (first list)
            hold (sess/pending session)
            slot (if (and (not head) (.isPresent hold)) (.get hold) nil)
            info (cond
                   head (task/brief head)
                   slot (pending/brief slot)
                   :else {})
            parsed (str (or (:topic info) ""))]
        (if (str/blank? parsed) "" parsed))
      topic)))
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
        text (if (seq items) "" (doctext/listify text))
        text (if (seq items) "" (doctext/normalize text))
        text (if (seq items) "" (doctext/rule text))
        text (if (seq items) "" (doccite/stars text))
        html (if (seq items)
               (str
                (if (str/blank? topic)
                  ""
                  (str "<p>" (doctext/escape topic) "</p>"))
                (doctext/outline items))
               (md/md-to-html-string text))
        html (if (seq items) html (doccite/tables html))
        html (if (seq items) html (doccite/codeindent html))
        html (if (seq items) html (doccite/backslash html))]
    (if (str/blank? html)
      ""
      (str "<div class=\"brief\"><div class=\"container\">"
           "<h1>Exploration Brief</h1>"
           "<div class=\"query\">"
           html
           "</div></div></div>"))))
(def heading "Heading text." doctext/heading)
(def normalize "List blank lines." doctext/normalize)
(def listify "Inline list conversion." doctext/listify)
(def paragraphs "List item paragraphs." doctext/paragraphs)
(def taskhtml "Task HTML section." doctasks/taskhtml)
(def tables "Table column classes." doccite/tables)
(def clean "Clean tracking params." doctext/clean)
(def trim "Trim url params." doctext/trim)
(def rule "Markdown rule to hr." doctext/rule)
(def citations "Citations to links." doccite/citations)
(def references "Reference links." doccite/references)
(def nested "List indent normalization." doctext/nested)
(def underscorify "Italic underscore pass." doctext/underscorify)
(def strip "Strip trailing sources." doccite/strip)
(def images "Append image block." docdata/images)
(def emojify "Wrap emoji spans." docsources/emojify)
(defn document
  "Create document instance."
  [session palette cover root]
  (->Document session palette cover root))
