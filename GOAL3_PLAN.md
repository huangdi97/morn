# GOAL3_PLAN.md — M0 → M9

## M0 Verify Goal 1/2 Baseline
运行 Goal 1/2 regression，读取 final reports、STATUS、DECISIONS、BLOCKERS、KNOWN_FAILURES，禁止重做已完成能力。

## M1 Certification Model
实现 CertificationSpec / Run / Decision / CertifiedWorkCapability / CapabilityRelease，并用 Goal 2 Evaluation/Replay/Shadow 作为证据输入。

## M2 Evolution Flywheel v0.2
实现 trace miner minimum、repetition detector、bottleneck detector、failure pattern detector、human correction aggregator、candidate generator、evidence window。先用 BioLab fixture/历史 run。

## M3 Deterministic Distillation
选择一个稳定 Actor step：Actor repeated behavior → candidate rule/program → regression → shadow → compare。实现 deterministic capability candidate + Actor fallback。

## M4 Managed Work / Outcome Delivery
实现 ManagedWorkService、DeliveryLifecycle、DeliveryReceipt、SLO、HumanFallback、RetryLiability、AcceptanceDecision。必须使用 Certified Capability。

## M5 Replacement Baseline
为一个具体 BioLab Work 建立 manual/existing baseline profile、observed process/work graph、baseline metrics、Morn orchestrated variant、Morn-native candidate。

## M6 Shadow Replace
相同输入下 baseline 与 candidate 隔离执行，输出 quality、cycle time、human effort、policy、cost、outcome、evidence。

## M7 Partial Replace Decision
只有 acceptance 不下降、critical safety/policy 不变差、evaluation/certification 通过、人类批准，才可产生 PartialReplaceCandidate。不自动退役旧系统。

## M8 Product Surfaces
更新 Evolution Center、Managed Work dashboard、Certification view、Replacement comparison、Capability release/history。

## M9 E2E + Closeout
跑完整主链，生成 `reports/goal3_final_report.md`。
