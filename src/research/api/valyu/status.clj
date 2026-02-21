(ns research.api.valyu.status
  (:require [jsonista.core :as json]
            [research.api.http :as request]
            [research.api.progress :as progress]))

(defprotocol Statused
  "Object that can fetch Valyu status."
  (status [item id] "Return status payload.")
  (pause [item span] "Pause before retry."))

(defrecord Status [base key data]
  Statused
  (pause [_ span]
    (Thread/sleep span))
  (status [item id]
    (let [url (str base "/deepresearch/tasks/" id "/status")
          head {"Content-Type" "application/json"
                "x-api-key" key}
          net (or (:net data) (request/make))
          log (or (:log data) (progress/make))
          limit 4
          span 1000]
      (loop [step 0]
        (let [result (try {:value @(request/get net url {:headers head
                                                         :timeout 60000
                                                         :as :text})}
                          (catch Exception exc
                            {:error exc}))
              reply (:value result)
              code (:status reply)
              body (:body reply)
              info (when (and code (< code 300) (some? body))
                     (json/read-value
                      body
                      (json/object-mapper {:decode-key-fn keyword})))
              signal (or (nil? code)
                         (>= code 500)
                         (= code 429)
                         (:error result))
              fault (some? (:error result))
              time (min (* span (inc step)) (* span 8))]
          (if info
            info
            (let [note (str "Valyu status non200 id="
                            id
                            " status="
                            (or code "none")
                            " attempt="
                            (inc step)
                            (when fault " error=true")
                            (when signal (str " wait_ms=" time)))]
              (println (progress/clean log note))
              (if signal
                (if (< step (dec limit))
                  (do (pause item time) (recur (inc step)))
                  (throw (ex-info (str "Valyu status failed id="
                                       id
                                       " status="
                                       (or code "none")
                                       " attempts="
                                       limit)
                                  {:id id
                                   :status code
                                   :attempts limit})))
                (throw (ex-info (str "Valyu status failed id="
                                     id
                                     " status="
                                     (or code "none"))
                                {:id id
                                 :status code}))))))))))

(defn make
  "Return status client."
  [base key data]
  (->Status base key data))
