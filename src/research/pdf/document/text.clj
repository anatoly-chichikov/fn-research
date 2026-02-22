(ns research.pdf.document.text
  (:require [clojure.string :as str]
            [research.api.response :as response]
            [research.domain.result :as result])
  (:import (org.jsoup Jsoup)
           (org.jsoup.parser Parser)))

(defn escape
  "Escape HTML special characters."
  [text]
  (-> text
      (str/replace "&" "&amp;")
      (str/replace "<" "&lt;")
      (str/replace ">" "&gt;")
      (str/replace "\"" "&quot;")))

(defn decode
  "Decode HTML entities."
  [text]
  (let [value (Parser/unescapeEntities (or text "") true)]
    (Parser/unescapeEntities value true)))

(defn heading
  "Return heading text with uppercase initial letter."
  [text]
  (let [text (str/trim (or text ""))
        size (count text)
        head (if (pos? size) (subs text 0 1) "")
        tail (if (> size 1) (subs text 1) "")
        head (str/upper-case head)]
    (str head tail)))

(defn slug
  "Return anchor slug."
  [text]
  (let [text (decode (str text))
        text (str/trim text)
        text (str/lower-case text)
        text (str/replace text #"[\s\p{Punct}]+" "-")
        text (str/replace text #"^-+|-+$" "")
        text (if (str/blank? text) "section" text)]
    text))

(defn anchor
  "Return unique anchor id."
  [_ seen]
  (let [idx (inc (get seen :idx 0))
        id (str "section-" idx)
        seen (assoc seen :idx idx)]
    [id seen]))

(defn anchors
  "Add anchor ids to headings and return html with toc items."
  [html]
  (let [doc (Jsoup/parseBodyFragment (str html))
        list (.select doc "h1, h2, h3, h4, h5, h6")
        data (reduce
              (fn [data node]
                (let [text (.text node)
                      [id seen] (anchor text (:seen data))
                      tag (.tagName node)
                      level (Integer/parseInt (subs tag 1))
                      back (str "<a class=\"toc-back\" href=\"#toc\" "
                                "aria-label=\"Back to contents\"></a>")]
                  (.attr node "id" id)
                  (when (= tag "h2")
                    (.prepend node back))
                  {:items (conj (:items data)
                                {:text text
                                 :id id
                                 :level level})
                   :seen seen}))
              {:items []
               :seen {}}
              list)]
    {:html (.html (.body doc))
     :items (:items data)}))

(defn toc
  "Render table of contents html."
  [items]
  (let [items (vec (or items []))
        top (filter (fn [item] (not= "Exploration Brief" (:text item))) items)
        base (if (seq top)
               (apply min (map :level top))
               (if (seq items) (apply min (map :level items)) 1))
        levels (set (map :level top))
        linklevels (cond
                     (contains? levels 1) #{1 2}
                     (contains? levels 2) #{2}
                     :else #{base})
        groups (loop [tail items head nil list []]
                 (if (seq tail)
                   (let [item (first tail)
                         level (:level item)
                         head? (= "Exploration Brief" (:text item))
                         root (or head? (contains? linklevels level))]
                     (if root
                       (recur (rest tail)
                              {:item item
                               :subs []}
                              (if head (conj list head) list))
                       (recur (rest tail)
                              (if head (update head :subs conj item) head)
                              list)))
                   (if head (conj list head) list)))
        rows (reduce
              (fn [text entry]
                (let [item (:item entry)
                      subs (:subs entry)
                      name (escape (str (:text item)))
                      id (escape (str (:id item)))
                      link (str "<a class=\"ref-link toc-row\" href=\"#"
                                id
                                "\">"
                                "<span class=\"toc-text\">"
                                name
                                "</span>"
                                "<span class=\"toc-page\" data-target=\"#"
                                id
                                "\"></span></a>")
                      subrows (reduce
                               (fn [text item]
                                 (let [name (escape (str (:text item)))
                                       id (escape (str (:id item)))
                                       row (str "<li class=\"toc-subitem\">"
                                                "<span class=\"toc-subtext\">"
                                                name
                                                "</span>"
                                                "<span class=\"toc-subpage\" "
                                                "data-target=\"#"
                                                id
                                                "\"></span></li>")]
                                   (str text row)))
                               ""
                               subs)
                      desc (if (str/blank? subrows)
                             ""
                             (str "<ul class=\"toc-sublist\">"
                                  subrows
                                  "</ul>"))
                      row (str "<li class=\"ref-item toc-item\">"
                               link
                               desc
                               "</li>")]
                  (str text row)))
              ""
              groups)]
    (if (str/blank? rows)
      ""
      (str "<div class=\"toc\" id=\"toc\"><div class=\"container\">"
           "<h1>Table of Contents</h1>"
           "<ul class=\"ref-list toc-list\">"
           rows
           "</ul></div></div>"))))

(defn normalize
  "Add blank lines before list markers."
  [text]
  (let [text (str/replace text #"\\n" "\n")
        rows (str/split text #"\n" -1)
        mark #"^\s*(?:[*+-] |\d+\. )"
        size (count rows)]
    (loop [idx 0 out []]
      (if (< idx size)
        (let [row (nth rows idx)
              prev (if (pos? idx) (nth rows (dec idx)) "")
              list (boolean (re-find mark row))
              back (boolean (re-find mark prev))
              blank (str/blank? prev)
              gap (and list (not back) (not blank) (pos? idx))]
          (recur (inc idx) (if gap (conj out "" row) (conj out row))))
        (str/join "\n" out)))))

(defn tablerows
  "Remove blank lines between markdown table rows."
  [text]
  (let [rows (str/split (str text) #"\n" -1)
        size (count rows)]
    (loop [idx 0 out [] past ""]
      (if (< idx size)
        (let [row (nth rows idx)
              tail (if (< (inc idx) size) (nth rows (inc idx)) "")
              blank (str/blank? row)
              pipe (re-find #"^\s*[|]" tail)
              lead (re-find #"^\s*[|]" past)
              dash (re-find #"^\s*[|]?[\s:-]*-[-|\s:]*$" past)
              skip (and blank pipe (or lead dash))
              out (if skip out (conj out row))
              past (if skip past row)]
          (recur (inc idx) out past))
        (str/join "\n" out)))))

(defn tablecite
  "Move trailing citations into the last table cell."
  [text]
  (let [rows (str/split (str text) #"\n" -1)
        rule (re-pattern "^(\\s*[|].*)[|]\\s*(\\[\\[\\d+\\]\\].*)$")]
    (loop [idx 0 out []]
      (if (< idx (count rows))
        (let [row (nth rows idx)
              hit (re-matches rule row)
              head (if hit (str/trimr (nth hit 1)) "")
              tail (if hit (nth hit 2) "")
              line (if hit (str head " " tail " |") row)]
          (recur (inc idx) (conj out line)))
        (str/join "\n" out)))))

(defn tablepipe
  "Ensure table rows end with pipe."
  [text]
  (let [rows (str/split (str text) #"\n" -1)]
    (loop [idx 0 out []]
      (if (< idx (count rows))
        (let [row (nth rows idx)
              head (re-find #"^\s*[|]" row)
              dash (re-matches #"^\s*[|]?[\s:-]*-[-|\s:]*$" row)
              line (if head
                     (if dash
                       (let [base (str/trimr row)]
                         (if (str/ends-with? base "|")
                           base
                           (str base "|")))
                       (if-let [hit (re-matches #"(.*?)[|]\s*$" row)]
                         (let [base (nth hit 1)
                               base (if (str/ends-with? base " ")
                                      base
                                      (str base " "))]
                           (str base "|"))
                         (let [base (if (str/ends-with? row " ")
                                      row
                                      (str row " "))]
                           (str base "|"))))
                     row)]
          (recur (inc idx) (conj out line)))
        (str/join "\n" out)))))

(defn tablelead
  "Remove list markers before table rows."
  [text]
  (let [rows (str/split (str text) #"\n" -1)
        rule #"^\s*[*+-]\s+([|].*)$"
        mark #"^\s*\d+[.)]\s+(\|.*)$"]
    (loop [idx 0 out []]
      (if (< idx (count rows))
        (let [row (nth rows idx)
              hit (re-matches rule row)
              alt (re-matches mark row)
              trim (str/triml row)
              line (cond
                     hit (nth hit 1)
                     alt (nth alt 1)
                     (and (not= row trim) (str/starts-with? trim "|")) trim
                     :else row)]
          (recur (inc idx) (conj out line)))
        (str/join "\n" out)))))

(defn listify
  "Convert inline prompts into markdown lists."
  [text]
  (let [text (str/replace text #"\\n" "\n")
        text (str/replace text #"\s+Research:" "\n\nResearch:")
        text (str/replace text #"(?m)(^|\n)(\s*)(\d+)\)" "$1$2$3.")
        text (str/replace text #"[ \t]+(\d+)[\.)]\s+" "\n$1. ")
        rows (str/split text #"\n" -1)
        rows (map (fn [row]
                    (if (re-find #"^\s*(?:\d+\.|[*+-])\s+" row)
                      row
                      (str/replace row #"[ \t]+([*+-])\s+" "\n$1 ")))
                  rows)
        text (str/join "\n" rows)
        text (str/replace text #"\n{3,}" "\n\n")]
    text))

(defn item
  "Normalize brief item."
  [node]
  (let [text (str (or (:text node) node))
        items (or (:items node) [])
        items (mapv item items)
        text (str/trim text)
        items (vec (remove
                    (fn [item]
                      (and (str/blank? (:text item))
                           (empty? (:items item))))
                    items))]
    {:text text
     :items items}))

(defn outline
  "Render nested list html."
  [items]
  (let [items (mapv item (or items []))
        rows (reduce
              (fn [list entry]
                (let [text (escape (str (or (:text entry) "")))
                      nest (outline (:items entry))
                      body (if (str/blank? nest) "" nest)
                      part (str "<li>" text body "</li>")]
                  (conj list part)))
              []
              items)
        body (str/join "" rows)]
    (if (str/blank? body) "" (str "<ol>" body "</ol>"))))

(defn rule
  "Convert markdown separators to hr tags."
  [text]
  (str/replace text #"\n---\n" "\n\n<hr />\n\n"))

(defn nested
  "Normalize list indent to four spaces."
  [text]
  (str/replace text #"(?m)^( {1,3})([*+-] )" "    $2"))

(defn paragraphs
  "Wrap list item text in paragraph tags."
  [html]
  (let [doc (Jsoup/parseBodyFragment html)
        list (.select doc "li")]
    (doseq [item list]
      (let [block (.select
                   item
                   (str "> p, > ul, > ol, > table, > pre, > div, > h1, "
                        "> h2, > h3, > h4, > h5, > h6, > blockquote"))
            body (.html item)]
        (when (and (zero? (.size block)) (not (str/blank? body)))
          (.html item (str "<p>" body "</p>")))))
    (.html (.body doc))))

(defn trim
  "Remove utm parameters from URL."
  [text]
  (try
    (let [part (java.net.URI. text)
          query (.getQuery part)
          value (if (and query (not (str/blank? query)))
                  (let [pairs (map #(str/split % #"=") (str/split query #"&"))
                        items (filter
                               (fn [item]
                                 (not
                                  (str/starts-with?
                                   (str/lower-case (first item))
                                   "utm_")))
                               pairs)
                        line (str/join "&" (map #(str/join "=" %) items))]
                    (if (= line query)
                      text
                      (str (.getScheme part)
                           "://"
                           (.getAuthority part)
                           (.getPath part)
                           (if (str/blank? line) "" (str "?" line))
                           (if (.getFragment part)
                             (str "#" (.getFragment part))
                             ""))))
                  text)]
      value)
    (catch Exception _ text)))

(defn presentation
  "Return decoded URL for display."
  [text]
  (let [text (str text)
        text (str/replace text "+" "%2B")]
    (try
      (java.net.URLDecoder/decode text "UTF-8")
      (catch Exception _ text))))

(defn prune
  "Remove utm fragments from text."
  [text]
  (str/replace text #"(\?utm_[^\s\)\]]+|&utm_[^\s\)\]]+)" ""))

(defn clean
  "Remove tracking parameters from text URLs."
  [text]
  (let [pattern #"https?://[^\s\)\]]+"
        value (str/replace text pattern (fn [link] (trim link)))
        value (prune value)
        mask (re-pattern
              (str "(?<!\\])[ \t]*\\("
                   "(?:https?://[^\\s\\)]+"
                   "|[A-Za-z0-9.-]+\\.[A-Za-z]{2,}[^\\s\\)]*)"
                   "\\)"))
        value (str/replace value mask "")]
    value))

(defn underscorify
  "Replace outer italic asterisks with underscores when bold ends the span."
  [text]
  (str/replace text #"(?<!\*)\*([^*\n]+?)\*\*([^\n]*?)\*\*\*" "_$1**$2**_"))

(defn label
  "Return cleaned source title."
  [item name]
  (let [text (str/replace
              (decode (str/trim (or (result/title item) "")))
              #"\s+"
              " ")
        link (trim (result/url item))
        host (response/domain link)
        text (if (and (= name "parallel")
                      (= (str/lower-case text) "fetched web page"))
               (if (str/blank? host) link host)
               text)
        text (if (str/blank? text) (if (str/blank? host) link host) text)]
    text))

(defn excerpt
  "Return cleaned excerpt text."
  [text]
  (let [text (decode (str/trim (or text "")))
        text (str/replace text #"\s+" " ")
        size 220
        text (if (> (count text) size)
               (str (subs text 0 (dec size)) "...")
               text)]
    text))
