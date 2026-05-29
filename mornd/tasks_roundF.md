# Morn — 轮F：24小时稳定性测试

## 目标
验证 Morn 核心进程可连续运行24小时无崩溃、无内存泄漏、所有子系统正常工作。

## 测试方案

### 模块F1：稳定性测试脚本

### 新文件
- `tests/stress_test.py` — 可独立运行的稳定性测试

### 规格
StressTest 类：

**监控指标**：
- 进程存活（每60秒检查一次）
- 内存使用（RSS，连续3次增长触发告警）
- 心跳计数（应持续增长）
- 情感状态（各维度在[0,1]范围内）
- 自省循环（light/deep 计数应持续增长）
- BondTracker 持久化（每轮交互后保存）
- 各后台循环（bond_update / drift / audit / self_prune）无异常退出

**模拟交互**：
- 每5分钟模拟一次对话（调用 ChatEngine 的 process_message）
- 每10次交互触发一次情感delta
- 每30分钟触发一次深度自省
- 每1小时触发一次 drift check
- 所有操作随机化（模拟真实使用）

**报告**：
- 每1小时输出进度和关键指标
- 结束时输出总结报告（运行时长、最大内存、心跳总数、是否异常退出）

### 运行方式
```bash
cd /home/hermes/morn/mornd
# 快速验证（5分钟）
python3 -m pytest tests/stress_test.py -v -x --timeout=600
# 完整测试（24小时）
python3 tests/stress_test.py
```

### 新增测试
`tests/test_stability_basics.py` — 6+测试
- 心跳循环不崩溃
- 内存监控不崩溃
- 情感状态始终在范围内
- 各后台循环可启动/停止

---

## 验收
1. 基础稳定性测试通过
2. 进程启动后各子系统初始化无异常
3. 情感状态始终在[0,1]
4. 心跳持续增长
