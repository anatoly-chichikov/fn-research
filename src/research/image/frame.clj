(ns research.image.frame
  (:require [research.image.generator :as gen])
  (:import (java.nio.file Files
                          LinkOption
                          Path
                          Paths
                          StandardCopyOption)
           (java.util ArrayList)
           (nu.pattern OpenCV)
           (org.opencv.core Core
                            Mat
                            MatOfDouble
                            MatOfFloat
                            MatOfInt
                            Rect
                            Size)
           (org.opencv.imgcodecs Imgcodecs)
           (org.opencv.imgproc Imgproc)))

(defprotocol Framed
  "Object that detects frames."
  (detect [item path] "Detect frame in image path.")
  (scan [item root] "Scan folder for cover images."))

(defn- loadlib
  "Load OpenCV native library."
  []
  (OpenCV/loadLocally)
  :ok)

(defn- fetch
  "Read image from path."
  [path]
  (let [mat (Imgcodecs/imread (.toString path) Imgcodecs/IMREAD_COLOR)]
    (if (or (nil? mat) (.empty mat))
      (throw
       (ex-info
        (str "Image read failed path=" (.toString path))
        {:path (.toString path)}))
      mat)))

(defn- resize
  "Return resized mat and scale."
  [mat span]
  (let [wide (.cols mat)
        high (.rows mat)
        size (double (max wide high))
        span (double span)
        ratio (if (> size span) (/ span size) 1.0)]
    (if (< ratio 1.0)
      (let [out (Mat.)
            w (int (Math/round (* wide ratio)))
            h (int (Math/round (* high ratio)))
            dims (Size. w h)]
        (Imgproc/resize mat out dims 0.0 0.0 Imgproc/INTER_AREA)
        {:mat out
         :scale ratio})
      {:mat mat
       :scale 1.0})))

(defn- median
  "Return median intensity."
  [gray]
  (let [hist (Mat.)
        channels (MatOfInt. (int-array [0]))
        mask (Mat.)
        bins (MatOfInt. (int-array [256]))
        range (MatOfFloat. (float-array [0.0 256.0]))]
    (try
      (let [list (ArrayList.)
            _ (.add list gray)
            _ (Imgproc/calcHist list channels mask hist bins range)
            total (* (.rows gray) (.cols gray))
            half (/ total 2.0)]
        (loop [idx 0 sum 0.0]
          (if (< idx 256)
            (let [cell (aget (.get hist idx 0) 0)
                  sum (+ sum cell)]
              (if (>= sum half) idx (recur (inc idx) sum)))
            0.0)))
      (finally
        (.release hist)
        (.release channels)
        (.release mask)
        (.release bins)
        (.release range)))))

(defn- canny
  "Return edge mat."
  [gray sigma floor]
  (let [mid (double (median gray))
        sigma (double sigma)
        low (double (max 0.0 (* (- 1.0 sigma) mid)))
        high (double (min 255.0 (* (+ 1.0 sigma) mid)))
        floor (double (or floor 0.0))
        low (if (pos? floor) (max low floor) low)
        high (if (pos? floor) (max high (* 2.0 floor)) high)
        edge (Mat.)]
    (Imgproc/Canny gray edge low high)
    edge))

(defn- strip
  "Return border rect."
  [side wide high width]
  (case side
    :top (Rect. 0 0 wide width)
    :bottom (Rect. 0 (- high width) wide width)
    :left (Rect. 0 0 width high)
    :right (Rect. (- wide width) 0 width high)))

(defn- inner
  "Return inner rect."
  [side wide high width]
  (case side
    :top (Rect. 0 width wide width)
    :bottom (Rect. 0 (- high (* 2 width)) wide width)
    :left (Rect. width 0 width high)
    :right (Rect. (- wide (* 2 width)) 0 width high)))

(defn- band
  "Return boundary rect."
  [side wide high width ring]
  (case side
    :top (Rect. 0
                (int (max 0 (- width ring)))
                wide
                (int (min high (+ ring ring 1))))
    :bottom (let [base (int (- high width 1))
                  low (int (max 0 (- base ring)))
                  top (int (min high (+ base ring 1)))
                  size (int (max 1 (- top low)))]
              (Rect. 0 low wide size))
    :left (Rect. (int (max 0 (- width ring)))
                 0
                 (int (min wide (+ ring ring 1)))
                 high)
    :right (let [base (int (- wide width 1))
                 low (int (max 0 (- base ring)))
                 top (int (min wide (+ base ring 1)))
                 size (int (max 1 (- top low)))]
             (Rect. low 0 size high))))

(defn- density
  "Return edge density."
  [edge rect]
  (let [part (.submat edge rect)
        mean (Core/mean part)
        value (/ (aget (.val mean) 0) 255.0)]
    (.release part)
    value))

(defn- stats
  "Return mean and std values."
  [lab rect]
  (let [part (.submat lab rect)
        mean (MatOfDouble.)
        std (MatOfDouble.)]
    (try
      (let [_ (Core/meanStdDev part mean std)
            mv (vec (.toArray mean))
            sv (vec (.toArray std))
            spread (/ (+ (nth sv 0) (nth sv 1) (nth sv 2)) 3.0)]
        {:mean mv
         :std spread})
      (finally
        (.release part)
        (.release mean)
        (.release std)))))

(defn- metrics
  "Return side metrics."
  [lab edge side width ring]
  (let [wide (.cols lab)
        high (.rows lab)
        size (min wide high)]
    (if (>= (* 2 width) size)
      nil
      (let [border (strip side wide high width)
            inner (inner side wide high width)
            band (band side wide high width ring)
            info (stats lab border)
            data (stats lab inner)
            mean (:mean info)
            core (:mean data)
            spread (:std info)
            gap (Math/sqrt
                 (+ (Math/pow (- (nth mean 0) (nth core 0)) 2.0)
                    (Math/pow (- (nth mean 1) (nth core 1)) 2.0)
                    (Math/pow (- (nth mean 2) (nth core 2)) 2.0)))
            rise (density edge band)
            noise (density edge border)
            score (/ (* gap rise) (+ spread 1.0E-6))]
        {:side side
         :mean mean
         :std spread
         :diff gap
         :edge rise
         :noise noise
         :score score}))))

(defn- match
  "Return true when metrics pass thresholds."
  [item config]
  (and (<= (:std item) (double (get config :std 8.0)))
       (>= (:diff item) (double (get config :diff 12.0)))
       (>= (:edge item) (double (get config :edge 0.25)))
       (<= (:noise item) (double (get config :noise 0.05)))))

(defn- tone
  "Return true when border tones match."
  [items limit]
  (let [tones (mapv :mean items)
        size (count tones)]
    (loop [i 0 top 0.0]
      (if (< i size)
        (let [a (nth tones i)
              top (loop [j (inc i) top top]
                    (if (< j size)
                      (let [b (nth tones j)
                            diff (Math/sqrt
                                  (+ (Math/pow
                                      (- (nth a 0) (nth b 0))
                                      2.0)
                                     (Math/pow
                                      (- (nth a 1) (nth b 1))
                                      2.0)
                                     (Math/pow
                                      (- (nth a 2) (nth b 2))
                                      2.0)))
                            top (max top diff)]
                        (recur (inc j) top))
                      top))]
          (recur (inc i) top))
        (<= top limit)))))

(defn- crest
  "Return line peak metrics."
  [edge side limit]
  (let [wide (.cols edge)
        high (.rows edge)
        limit (int (max 1
                        (min limit
                             (if (or (= side :top) (= side :bottom))
                               high
                               wide))))
        base (case side
               :top 0
               :bottom (- high limit)
               :left 0
               :right (- wide limit))
        span (if (or (= side :top) (= side :bottom)) wide high)]
    (loop [idx 0 top 0.0 sum 0.0]
      (if (< idx limit)
        (let [pos (+ base idx)
              rect (if (or (= side :top) (= side :bottom))
                     (Rect. 0 pos span 1)
                     (Rect. pos 0 1 span))
              val (density edge rect)
              top (max top val)
              sum (+ sum val)]
          (recur (inc idx) top sum))
        (let [mean (/ sum limit)
              ratio (if (pos? mean) (/ top mean) top)]
          {:side side
           :peak top
           :mean mean
           :ratio ratio})))))

(defn- line
  "Return line detection data."
  [edge limit ridge peak sides]
  (let [list (mapv (fn [side] (crest edge side limit))
                   [:top :bottom :left :right])
        good (vec
              (filter
               (fn [item]
                 (and (>= (:peak item) ridge)
                      (>= (:ratio item) peak)))
               list))
        names (mapv :side good)
        ok (>= (count names) sides)]
    {:ok ok
     :sides names}))

(defn- limit
  "Return max border width."
  [wide high cap base]
  (let [size (min wide high)
        raw (int (Math/floor (* size cap)))
        raw (max raw base)
        raw (min raw (quot size 4))]
    raw))

(defn- files
  "Return cover image paths under root."
  [root lead exts]
  (with-open [stream (Files/walk
                      root
                      (make-array java.nio.file.FileVisitOption 0))]
    (let [paths (iterator-seq (.iterator stream))
          paths (filter (fn [path]
                          (let [name (.toString (.getFileName path))
                                low (.toLowerCase name)
                                dot (.lastIndexOf low ".")
                                ext (if (neg? dot) "" (subs low dot))]
                            (and (Files/isRegularFile
                                  path
                                  (make-array LinkOption 0))
                                 (.startsWith low lead)
                                 (contains? exts ext))))
                        paths)]
      (vec (sort-by (fn [path] (.toString path)) paths)))))

(defrecord Detector [config]
  Framed
  (detect [_ path]
    (let [path (if (instance? Path path)
                 path
                 (Paths/get (str path) (make-array String 0)))
          ok (Files/exists path (make-array LinkOption 0))
          _ (when-not ok
              (throw
               (ex-info
                (str "Image path missing path=" (.toString path))
                {:path (.toString path)})))
          raw (fetch path)
          span (double (get config :span 1024.0))
          data (resize raw span)
          mat (:mat data)
          rate (:scale data)
          lab (Mat.)
          gray (Mat.)]
      (try
        (let [_ (Imgproc/cvtColor mat lab Imgproc/COLOR_BGR2Lab)
              _ (Imgproc/cvtColor mat gray Imgproc/COLOR_BGR2GRAY)
              floor (double (get config :floor 0.0))
              edge (canny
                    gray
                    (double (get config :sigma 0.33))
                    floor)]
          (try
            (let [wide (.cols gray)
                  high (.rows gray)
                  cap (double (get config :cap 0.05))
                  base (int (get config :min 1))
                  ring (int (get config :band 1))
                  sides (int (get config :sides 4))
                  bound (double (get config :tone 15.0))
                  limit (limit wide high cap base)
                  best (loop [w base
                              best {:ok false
                                    :score 0.0
                                    :width 0
                                    :sides []
                                    :tone bound}]
                         (if (<= w limit)
                           (let [list (mapv
                                       (fn [side]
                                         (metrics lab edge side w ring))
                                       [:top :bottom :left :right])
                                 list (vec (remove nil? list))
                                 good (vec (filter
                                            (fn [node]
                                              (match node config))
                                            list))
                                 ok (>= (count good) sides)
                                 keep (and ok (tone good bound))
                                 score (if keep
                                         (apply min (mapv :score good))
                                         0.0)
                                 best (if (> score (:score best))
                                        {:ok keep
                                         :score score
                                         :width w
                                         :sides (mapv :side good)
                                         :tone bound}
                                        best)]
                             (recur (inc w) best))
                           best))
                  ridge (double (get config :ridge 0.35))
                  peak (double (get config :peak 3.0))
                  line (line edge limit ridge peak sides)
                  mode (cond (and (:ok best) (:ok line)) "both"
                             (:ok best) "metrics"
                             (:ok line) "line"
                             :else "none")
                  frame (not= mode "none")
                  info {:mode mode
                        :scale rate
                        :width (:width best)
                        :sides (:sides best)
                        :line (:sides line)
                        :tone (:tone best)}]
              {:frame frame
               :info info})
            (finally
              (.release edge))))
        (finally
          (.release raw)
          (when-not (identical? raw mat) (.release mat))
          (.release lab)
          (.release gray)))))
  (scan [item root]
    (let [root (if (instance? Path root)
                 root
                 (Paths/get (str root) (make-array String 0)))
          ok (Files/exists root (make-array LinkOption 0))
          _ (when-not ok
              (throw
               (ex-info
                (str "Root path missing path=" (.toString root))
                {:path (.toString root)})))
          lead (str (get config :lead "cover"))
          exts (set (get config :exts #{".png" ".jpg" ".jpeg" ".webp"}))
          list (files root (.toLowerCase lead) exts)
          rows (mapv
                (fn [path]
                  (let [data (detect item path)]
                    {:path (.toString path)
                     :frame (:frame data)
                     :info (:info data)}))
                list)
          total (count rows)
          hits (count (filter :frame rows))]
      {:total total
       :hits hits
       :rows rows})))

(defn detector
  "Create frame detector."
  ([] (detector {}))
  ([config]
   (let [base {:cap 0.05
               :min 1
               :std 8.0
               :diff 12.0
               :edge 0.25
               :noise 0.05
               :sides 4
               :tone 15.0
               :span 1024.0
               :sigma 0.33
               :floor 0.0
               :ridge 0.35
               :peak 3.0
               :band 1
               :lead "cover"
               :exts #{".png" ".jpg" ".jpeg" ".webp"}}
         config (merge base config)
         _ (loadlib)]
     (->Detector config))))

(defn- attempt
  "Return attempt path."
  [path step]
  (let [file (.toString (.getFileName path))
        dot (.lastIndexOf file ".")
        stem (if (neg? dot) file (subs file 0 dot))
        tail (if (neg? dot) "" (subs file dot))
        name (str stem ".attempt-" step tail)
        root (.getParent path)]
    (.resolve root name)))

(defn- backup
  "Copy cover to attempt path."
  [path step]
  (let [target (attempt path step)]
    (Files/copy
     path
     target
     (into-array
      java.nio.file.CopyOption
      [StandardCopyOption/REPLACE_EXISTING]))
    target))

(defn retry
  "Regenerate image until frame is gone or limit reached."
  [generator detector topic path limit]
  (let [name (.toString (.getFileName path))
        limit (int limit)
        result (do (gen/generate generator topic path)
                   (detect detector path))]
    (if (:frame result)
      (loop [step 1]
        (backup path step)
        (println
         (str "[FRAME_DETECTED] "
              name
              " — "
              "Oops... Gemini generated a cover with a frame. Will regenerate"))
        (println
         (str "[REGENERATED] "
              name
              " — attempt "
              step
              "/"
              limit))
        (gen/generate generator topic path)
        (let [value (detect detector path)]
          (if (:frame value)
            (if (< step limit)
              (recur (inc step))
              (do (println
                   (str "[FAILED] "
                        name
                        " — frame still present after "
                        limit
                        " attempts"))
                  {:frame true
                   :tries limit
                   :info (:info value)}))
            (do (println
                 (str "[SUCCESS] "
                      name
                      " — frame removed after "
                      step
                      " attempts"))
                {:frame false
                 :tries step
                 :info (:info value)}))))
      {:frame false
       :tries 0
       :info (:info result)})))
