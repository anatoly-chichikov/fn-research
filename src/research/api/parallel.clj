(ns research.api.parallel
  (:require [clojure.java.io :as io]
            [clojure.string :as str]
            [jsonista.core :as json]
            [research.api.http :as request]
            [research.api.parallel.support :as support]
            [research.api.progress :as progress]
            [research.api.research :as research]
            [research.api.response :as response]
            [research.config :as config]))

(defn now
  "Return current time millis."
  []
  (support/now))

(defn env
  "Return environment value by key."
  [key]
  (support/env key))

(defn clean
  "Remove periods from log text."
  [text]
  (support/clean text))

(defn emit
  "Emit progress event to stdout."
  [log data]
  (support/emit log data))

(defn parse
  "Parse SSE data payload into map."
  [text]
  (support/parse text))

(defn sse
  "Stream SSE events from reader."
  [reader log]
  (support/sse reader log))

(defrecord Parallel [key base time]
  research/Researchable
  (start [_ query processor]
    (let [url (str base "/v1/tasks/runs")
          spec (str "Include as many details"
                    " from collected sources as possible.")
          body {:input query
                :processor processor
                :enable_events true
                :task_spec {:output_schema {:type "text"
                                            :description spec}}}
          head {"x-api-key" key
                "Content-Type" "application/json"
                "parallel-beta" "events-sse-2025-07-24"}
          net (:net time)
          response @(request/post net url {:headers head
                                           :body
                                           (json/write-value-as-string body)
                                           :timeout 60000
                                           :as :text})
          status (:status response)
          data (if (< status 300)
                 (json/read-value
                  (:body response)
                  (json/object-mapper {:decode-key-fn keyword}))
                 (throw (ex-info "Parallel create failed" {:status status})))
          run (or (:run_id data) (get-in data [:run :run_id]) "")]
      run))
  (stream [_ id]
    (let [url (str base "/v1beta/tasks/runs/" id "/events")
          head {"x-api-key" key
                "Accept" "text/event-stream"
                "parallel-beta" "events-sse-2025-07-24"}
          net (:net time)
          response @(request/get net url {:headers head
                                          :as :stream
                                          :timeout 60000})
          body (:body response)
          log (:log time)]
      (if body (with-open [reader (io/reader body)] (sse reader log)) true)))
  (finish [_ id]
    (let [url (str base "/v1/tasks/runs/" id "/result")
          head {"x-api-key" key
                "Content-Type" "application/json"}
          timeout-sec (* config/task-timeout-hours 3600)
          timeout-ms (* config/task-timeout-hours 3600000)
          net (:net time)
          response @(request/get net url {:headers head
                                          :query-params
                                          {:api_timeout timeout-sec}
                                          :timeout timeout-ms
                                          :as :text})
          status (:status response)
          raw (if (= status 200)
                (json/read-value
                 (:body response)
                 (json/object-mapper {:decode-key-fn keyword}))
                (throw (ex-info "Parallel result failed" {:status status})))
          output (:output raw)
          text (if (map? output) (or (:content output) "") "")
          basis (if (map? output) (or (:basis output) []) [])
          run (or (:run raw) {})
          code (if (map? run) (or (:run_id run) id) id)
          state (if (map? run) (or (:status run) "completed") "completed")]
      (response/response {:id code
                          :status state
                          :output text
                          :basis basis
                          :raw raw}))))

(defn parallel
  "Create Parallel client from env."
  []
  (let [key (or (env "PARALLEL_API_KEY") "")
        base (or (env "PARALLEL_BASE_URL") "https://api.parallel.ai")
        data {:log (progress/make)
              :net (request/make)}]
    (if (str/blank? key)
      (throw (ex-info "PARALLEL_API_KEY is required" {}))
      (->Parallel key base data))))
