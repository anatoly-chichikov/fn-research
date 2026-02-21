(ns research.api.valyu
  (:require [clojure.string :as str]
            [jsonista.core :as json]
            [research.api.http :as request]
            [research.api.link :as link]
            [research.api.progress :as progress]
            [research.api.research :as research]
            [research.api.response :as response]
            [research.api.valyu.status :as status]
            [research.config :as config]))

(declare valyu-emit)

(defn message
  "Return newest message and updated seen map."
  [value seen token]
  (let [items (or (:messages value) [])
        size (get seen token 0)]
    (if (<= (count items) size)
      ["" seen]
      (let [item (last items)
            text (or (:message item) (:content item) (:text item) "")
            text (cond
                   (vector? text) (str/join " " (map str text))
                   (map? text) (str text)
                   :else (str text))
            next (assoc seen token (count items))]
        [text next]))))

(defrecord Valyu [key base data]
  research/Researchable
  (start [_ query processor]
    (let [url (str base "/deepresearch/tasks")
          body {:input query
                :model processor
                :output_formats ["markdown" "pdf"]}
          head {"Content-Type" "application/json"
                "x-api-key" key}
          net (:net data)
          response @(request/post net url {:headers head
                                           :body
                                           (json/write-value-as-string body)
                                           :timeout 60000
                                           :as :text})
          status (:status response)
          info (if (< status 300)
                 (json/read-value
                  (:body response)
                  (json/object-mapper {:decode-key-fn keyword}))
                 (throw (ex-info "Valyu create failed" {:status status})))
          run (or (:deepresearch_id info) (:id info) "")]
      run))
  (stream [item id]
    (let [timeout-ms (* config/task-timeout-hours 3600000)
          log (:log (:data item))
          unit (:state (:data item))
          info (loop [start (System/currentTimeMillis)]
                 (let [data (status/status unit id)
                       state (or (:status data) "")
                       value (if (map? state) (or (:value state) state) state)
                       done (or (= value "completed")
                                (= value "failed")
                                (= value "cancelled")
                                (= value "canceled"))
                       _ (valyu-emit log data)]
                   (if done
                     data
                     (if (> (- (System/currentTimeMillis) start) timeout-ms)
                       (throw (ex-info "Valyu task timed out" {:id id}))
                       (do (status/pause unit 180000) (recur start))))))]
      info))
  (finish [item id]
    (let [unit (:state (:data item))
          data (status/status unit id)
          output (:output data)
          text (if (map? output)
                 (or (:markdown output) (:content output) "")
                 (or output ""))
          sources (or (:sources data) [])
          base (research/basis item sources)
          state (or (:status data) "completed")
          status (if (map? state) (or (:value state) state) state)
          code (or (:deepresearch_id data) (:id data) id)]
      (response/response {:id code
                          :status status
                          :output text
                          :basis base
                          :raw data})))
  research/Grounded
  (basis [_ sources]
    (let [policy (link/make)]
      (reduce
       (fn [list data]
         (let [url (or (:url data) "")
               text (or (:content data) (:snippet data) (:description data) "")
               title (or (:title data)
                         (if (str/blank? url) "" (link/domain policy url)))]
           (if (str/blank? url)
             list
             (conj list {:citations [{:title title
                                      :url url
                                      :excerpts [text]}]}))))
       []
       sources))))

(defn valyu-emit
  "Emit progress info for Valyu."
  [log data]
  (let [status (or (:status data) "")
        progress (or (:progress data) {})
        current (get progress :current_step nil)
        total (get progress :total_steps nil)
        message (or (:message data) "")
        items (cond-> []
                (not (str/blank? (str status)))
                (conj (str status))
                (and (some? current) (some? total))
                (conj (str current "/" total))
                (not (str/blank? message))
                (conj message))
        line (if (seq items) (str/join " | " items) (str data))]
    (progress/emit log (str "[PROGRESS] " line))))

(defn valyu
  "Create Valyu client from env or map."
  [item]
  (let [key (or (:key item) (System/getenv "VALYU_API_KEY") "")
        base (or (:base item)
                 (System/getenv "VALYU_BASE_URL")
                 "https://api.valyu.ai")
        base (if (and (str/includes? base "api.valyu.ai")
                      (not (str/ends-with? base "/v1")))
               (str (str/replace base #"/+$" "") "/v1")
               base)
        mode (or (:mode item) "")
        data {:log (progress/make)
              :net (request/make)}
        unit (status/make base key {:log (:log data)
                                    :net (:net data)})
        unit (or (:state item) unit)]
    (if (and (str/blank? key) (not= mode "basis"))
      (throw (ex-info "VALYU_API_KEY is required" {}))
      (->Valyu key base {:log (:log data)
                         :net (:net data)
                         :state unit}))))
