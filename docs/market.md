# 市场文档

## Marketplace 是什么

`src/market/marketplace.rs::Marketplace` 是组件 / Agent / 工作流的分发市场。支持商品搜索、上架、购买、许可证管理、评分。

## Listing（上架商品）

```rust
pub struct Listing {
    pub id: String,
    pub item_type: String,  // tool / knowledge / skill / persona / agent / workflow
    pub name: String,
    pub description: String,
    pub price: f64,
    pub author: String,
    pub rating: f64,
    pub downloads: u64,
    pub created_at: String,
}
```

系统内置 6 个 Listing：

| 名称 | 类型 | 价格 | 作者 |
|------|------|------|------|
| Web Search Pro | tool | ¥0.001 | Morn Labs |
| Stock Market Data | knowledge | ¥0.01 | Morn Labs |
| Deep Research Skill | skill | ¥0.01 | Morn Labs |
| Financial Analyst | persona | ¥0.00 | Morn Labs |
| Research Agent | agent | ¥0.05 | Morn Labs |
| Weekly Report Generator | workflow | ¥0.03 | Morn Labs |

主要 API：

| 方法 | 功能 |
|------|------|
| `list(filter)` | 列出商品，可选按类型过滤 |
| `get(id)` | 获取单个商品 |
| `search(query)` | 按名称 / 描述 / 标签搜索 |
| `publish(listing)` | 上架新商品 |
| `purchase(listing_id, user_id)` | 购买，返回 License |
| `install(listing_id, user_id)` | 安装已购买商品 |
| `rate(listing_id, user_id, score, review)` | 评分 |

## Transaction（交易）

`Transaction` 记录每次购买的详情：

```rust
pub struct Transaction {
    pub id: String,
    pub listing_id: String,
    pub buyer: String,
    pub amount: f64,
    pub timestamp: String,
}
```

## License（许可证管理）

购买后生成 `License`，可直接调用 `install` 验证：

```rust
pub struct License {
    pub id: String,
    pub listing_id: String,
    pub user_id: String,
    pub granted_at: String,
    pub expires_at: Option<String>,  // 免费商品无过期
}
```

## 评分与下载

评分采用加权平均算法：

```rust
listing.rating = (rating * downloads + score) / (downloads + 1);
listing.downloads += 1;
```