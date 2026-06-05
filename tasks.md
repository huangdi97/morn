# Morn Market 交易流 — 任务清单

## 核心准则 (编程原则)

### 14条核心准则

1. **Think Before Coding** — 阅读整个任务文件再动手，理解现有代码结构
2. **The Code Works** — 每次修改后 `cargo build` 通过，`cargo test` 全绿
3. **Small Batches** — 改一个模块编译一次，不堆砌批量修改
4. **No Dead Code** — 不添加未使用的函数或注释
5. **Single Source of Truth** — 数据库结构以代码中的 schema 为准
6. **Test the Paths** — 新增功能必须加测试覆盖
7. **Fail Fast** — 遇到编译错误立即停止并输出
8. **Leave It Better** — 代码格式统一，遵守现有风格
9. **Never Guess the Stack** — 从实际代码中理解现有 API
10. **Read Before You Write** — 先读 marketplace.rs、storage.rs、cli.rs 再动手
11. **Prefer Friction Logs** — 记录构建中遇到的问题
12. **Respect the Dependency** — marketplace 依赖 storage 模块
13. **No Ambiguous Names** — 变量/函数命名清晰
14. **Document Decisions, Not Drama** — 写清晰的代码注释

### 低耦合3条

1. **模块独立** — marketplace 模块只依赖 core::storage，不依赖 CLI 模块
2. **接口一致** — CLI 命令风格与现有 `/help`、`/status` 一致
3. **渐进增强** — 不破坏现有测试，加法优先

### 执行规则

- 按任务顺序执行
- 每个任务后 `cargo build && cargo test`
- 以 `~/morn-desktop/` 为根目录
- 最终 git add/commit/push 到 main

---

## 前置阅读

开始前先读以下文件了解现有结构：
- `src/market/marketplace.rs` — 现有 Market 实现
- `src/core/storage.rs` — SQLite 存储模块
- `src/channel/cli.rs` — CLI 通道，需添加市场命令
- `src/core/registry.rs` — 能力注册中心，安装需要这里

---

## 任务列表

### 任务1: Storage 增加市场表

在 `src/core/storage.rs` 中增加 3 张表：
- `market_listings` — Listing 数据（id, item_type, name, description, price, author, rating, downloads, created_at）
- `market_transactions` — 交易记录（id, listing_id, buyer, amount, timestamp）
- `market_licenses` — 许可证（id, listing_id, user_id, granted_at, expires_at）

添加对应的 CRUD 方法：
- `save_listing(&self, listing: &Listing) -> Result<(), String>`
- `list_listings(&self, filter: Option<&str>) -> Result<Vec<Listing>, String>`
- `get_listing(&self, id: &str) -> Result<Option<Listing>, String>`
- `save_transaction(&self, tx: &Transaction) -> Result<(), String>`
- `save_license(&self, lic: &License) -> Result<(), String>`
- `get_user_licenses(&self, user_id: &str) -> Result<Vec<License>, String>`
- `update_listing_rating(&self, id: &str, rating: f64, downloads: u64) -> Result<(), String>`
- `delete_listing(&self, id: &str) -> Result<(), String>`

注意：Listing/Transaction/License 定义在 marketplace.rs 中，storage.rs 需要引用它们。
为避免循环依赖，在 `market/` 模块中定义数据结构，storage.rs 引用。

### 任务2: Marketplace 对接 Storage

修改 `src/market/marketplace.rs`：
- 构造函数 `Marketplace::new(storage: Storage)` — 从 storage 加载数据
- `list_builtin()` 静态方法 — 创建内置 Listing 并写入 storage（如果不存在）
- 所有方法改为通过 Storage 操作，而非 HashMap
- 保留方法签名不变（list/get/purchase/publish/rate/search/install/transactions/user_licenses）

### 任务3: CLI 增加市场命令

修改 `src/channel/cli.rs`：
- 添加 `/market` 子命令系列：
  - `/market list` — 列出市场上所有商品（可加类型过滤：`/market list tool`）
  - `/market show <id>` — 查看商品详情
  - `/market buy <id>` — 购买商品
  - `/market install <id>` — 安装已购买的商品
  - `/market search <query>` — 搜索商品
  - `/market my` — 查看我的购买/许可证
  - `/market publish <id>` — 发布组件到市场
- 保持现有 `/help` 命令更新，增加市场命令说明

### 任务4: 购买 → 安装全链路

实现购买后的实际安装逻辑：
- `purchase()` 创建 Transaction + License，写入 storage
- `install()` 检查 License 后，将商品对应的能力注册到 Registry
- 实现一个 `install_to_registry()` 私有方法，将 Listing 转换为注册项

### 任务5: 测试

为新增功能写测试：
- Storage 市场表 CRUD 测试
- Market 持久化后的购买/安装/评分集成测试
- CLI 市场命令的基本测试（mock 不需要真实的终端交互）

### 任务6: 清理与提交

```bash
cd ~/morn-desktop
cargo build 2>&1
cargo test 2>&1
git add -A
git commit -m "feat: market transaction flow with SQLite persistence

- Add market tables (listings/transactions/licenses) to Storage
- Wire Marketplace to SQLite storage
- Add /market CLI commands (list/show/buy/install/search/my/publish)
- Implement purchase→install full pipeline with Registry integration"
git push origin main
```
