(ns research.image.generator
  (:require [clojure.edn :as edn]
            [clojure.java.io :as io]
            [clojure.string :as str]
            [jsonista.core :as json]
            [org.httpkit.client :as http])
  (:import (java.io ByteArrayInputStream)
           (java.nio.file Files)
           (java.nio.file.attribute FileAttribute)
           (java.util Base64)))

(defprotocol Generated
  "Object that can generate images."
  (generate [item topic path] "Generate image at path."))

(defn prompt
  "Return prompt with topic inserted."
  [text topic]
  (str/replace text "%s" topic))

(defn image
  "Extract image bytes from response."
  [data]
  (let [items (or (:candidates data) [])
        item (first items)
        parts (get-in item [:content :parts] [])
        part (first parts)
        inline (or (:inlineData part) (:inline_data part) {})
        value (or (:data inline) "")]
    (if (str/blank? value) (byte-array 0) (.decode (Base64/getDecoder) value))))

(defn compress
  "Compress image bytes into jpeg."
  [data path quality]
  (let [file (.toFile path)
        _ (Files/createDirectories
           (.getParent path)
           (make-array FileAttribute 0))
        writers (javax.imageio.ImageIO/getImageWritersByFormatName "jpeg")
        writer (if (.hasNext writers)
                 (.next writers)
                 (throw (ex-info "JPEG writer missing" {})))
        param (.getDefaultWriteParam writer)]
    (try
      (with-open [input (ByteArrayInputStream. data)
                  output (javax.imageio.ImageIO/createImageOutputStream file)]
        (let [image (javax.imageio.ImageIO/read input)]
          (.setCompressionMode
           param
           javax.imageio.ImageWriteParam/MODE_EXPLICIT)
          (.setCompressionQuality param (float quality))
          (.setOutput writer output)
          (.write writer nil (javax.imageio.IIOImage. image nil nil) param)
          path))
      (finally
        (.dispose writer)))))

(defrecord Generator [key spec data]
  Generated
  (generate [_ topic path]
    (let [model (:model data)
          url (str "https://generativelanguage.googleapis.com/v1beta/models/"
                   model
                   ":generateContent?key="
                   key)
          body {:contents [{:parts [{:text (prompt spec topic)}]}]
                :generationConfig {:responseModalities ["IMAGE"]
                                   :imageConfig {:aspectRatio "16:9"
                                                 :imageSize "1K"}}}
          head {"Content-Type" "application/json"}
          response @(http/post url {:headers head
                                    :body (json/write-value-as-string body)
                                    :timeout 600000
                                    :as :text})
          status (:status response)
          raw (if (and status (< status 300))
                (json/read-value
                 (:body response)
                 (json/object-mapper {:decode-key-fn keyword}))
                (throw (ex-info
                        (str "Gemini image failed model="
                             model
                             " status="
                             (or status "none"))
                        {:status status
                         :model model})))
          value (image raw)]
      (if (zero? (alength value))
        (throw (ex-info "Gemini image missing" {}))
        (compress value path (:quality data))))))

(defn generator
  "Create generator from env."
  []
  (let [key (or (System/getenv "GEMINI_API_KEY") "")
        root (io/resource "cover/parts.edn")
        text (if root
               (slurp root)
               (throw (ex-info "Cover parts missing"
                               {:resource "cover/parts.edn"})))
        data (edn/read-string text)
        topic (or (:topic data) "")
        node (io/resource topic)
        value (let [entry (if node
                            (edn/read-string (slurp node))
                            (throw (ex-info "Cover topic missing"
                                            {:resource topic})))
                    text (get entry :topic)]
                (if (str/blank? (or text ""))
                  (throw (ex-info "Cover topic missing" {:resource topic}))
                  text))
        items (or (:image data) [])
        image (reduce
               (fn [result item]
                 (let [path (io/resource item)
                       value (if path
                               (slurp path)
                               (throw (ex-info "Cover part missing"
                                               {:resource item})))
                       entry (edn/read-string value)]
                   (merge result entry)))
               {}
               items)
        spec {:topic value
              :marketing_image image}
        body (json/write-value-as-string spec)
        data {:model "gemini-3-pro-image-preview"
              :quality 0.85}]
    (if (str/blank? key)
      (throw (ex-info "GEMINI_API_KEY is required" {}))
      (->Generator key body data))))
