# GOAL5_DOMAIN_NEUTRALIZATION.md

## Audit
搜索 code + persisted contracts + UI：`biolab`, `scientific`, `hypothesis`, `experiment`, `manuscript` 以及真实仓库发现的其他领域概念。不要盲删 generic infrastructure 中合理出现的 dataset 等词，判断语义所有权。

分类：
- CORE_BUG：具体领域行为进入 Core
- NEEDS_EXTRACTION：实现有用但应移入 pack/reference
- REFERENCE_ONLY：已正确隔离
- TEST_FIXTURE：仅测试使用，优先迁到 conformance/reference
- SAFE_GENERIC：真正领域无关

必须证明：
1. Core build without reference pack
2. Core tests without reference pack
3. app starts with zero domain packs
4. reference pack 可 validate/build/install/enable
5. 领域 UI/类型只在 enable 后出现
6. disable/uninstall 后 Core 健康
7. 历史 records/provenance 仍可读
