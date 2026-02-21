(ns research.pdf.document
  (:require [clojure.string :as str]
            [research.domain.session :as sess]
            [research.pdf.document.citations :as doccite]
            [research.pdf.document.data :as docdata]
            [research.pdf.document.env :as docenv]
            [research.pdf.document.sources :as docsources]
            [research.pdf.document.tasks :as doctasks]
            [research.pdf.document.text :as doctext]
            [research.pdf.style :as style]))
(declare author service coverimage brief emit)
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
           (doctext/escape (doctext/heading (sess/topic session)))
           "</title><style>"
           css
           "</style></head><body>"
           "<div class=\"page-footer\">"
           note
           "</div>"
           "<div class=\"intro\">"
           (coverimage item)
           "<div class=\"intro-content\"><h1>"
           (doctext/escape (doctext/heading (sess/topic session)))
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
(def service "Service name." docenv/service)
(def coverimage "Cover image html." docenv/coverimage)
(def brief "Brief section." docenv/brief)
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
