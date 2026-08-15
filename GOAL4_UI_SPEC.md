# Goal4 UI Spec

Workbench：predicted duration/ETA、failure risk、cost、human intervention、acceptance/outcome、uncertainty、context status、predictor version；run 后并排 Prediction-at-start / Actual / Error。

Studio：Candidate A/B/C 同屏比较 predicted outcome/risk/duration/cost/human load/uncertainty + replay/simulation + gaps；禁止只有一个 AI 综合分。

Console：migration/repository health、Predictor Registry、dataset/version、evaluation/calibration/drift/context、Rollback requests/receipts。

Evolution Center：expected delta、uncertainty、actual delta、prediction error、replay/eval/shadow/certification。

若真实 DSH/真实数据 blocked，UI 必须显示 External verification pending，不能绿色假完成。
