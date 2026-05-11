# ADR 003 — 多链路由决策器

> 日期：2026-05-11 | 状态：draft | 作者：qc + DeepSeek

---

## 决策

路由评分公式：

```
得分 = 速度系数 × 0.6 + 成本系数 × 0.3 + 安全系数 × 0.1
```

**默认速度优先**（用户可切成本优先）。

## 评分系数

### 速度系数 (0.6)

```
输入: 出块时间 (block_time) + 交易池拥堵度 (mempool_load)
公式: 1 / (1 + mempool_load × block_time_norm)
  快链 (block_time=0.4s, load=0.2) → 0.93
  慢链 (block_time=3s, load=0.8)   → 0.29
```

### 成本系数 (0.3)

```
输入: Gas Price + 跨链桥 fee
公式: 1 / (1 + gas_price_norm × 0.5 + bridge_fee_norm × 0.5)
  低 Gas 链 → 趋近 1
  高 Gas 链 → 趋近 0
```

### 安全系数 (0.1)

```
输入: 24h 异常事件 (reorg > 1block, 桥失败率)
公式: 1 - (reorg_event × 0.5 + bridge_fail_rate)
  稳定链 → 1.0
  有 reorg → 0.5
  桥故障 → 0
```

## 桥白名单

```
主: Stargate (LayerZero)  — 统一接口，覆盖率最高
备: CBridge (Celer)        — fallback，仅 Stargate 不可用时
禁: 未审计小桥 / 资金池不明 / 无 7d+ 无故障记录
```

白名单硬编码，不动态发现。新增桥需 ADR 审批 + 安全审计。

## Router 实现 (Rust)

```rust
use std::sync::Arc;

struct ChainScore {
    chain_id: ChainId,
    block_time: Duration,
    gas_price: f64,
}

impl ChainScore {
    fn score(&self, tx: &Tx) -> f64 {
        let speed = 1.0 / (1.0 + self.mempool_load() * self.block_time.norm());
        let cost  = 1.0 / (1.0 + self.gas_price.norm() * 0.5);
        let safety = 1.0 - self.recent_reorg_penalty();
        speed * 0.6 + cost * 0.3 + safety * 0.1
    }
}

impl Router {
    pub fn route(&self, tx: &Tx) -> ChainId {
        self.chains.iter()
            .filter(|c| self.bridge_whitelist.contains(&c.chain_id))
            .map(|c| (c.score(tx), c.chain_id))
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(_, id)| id)
            .unwrap_or(FALLBACK_CHAIN)
    }
}
```

## Oracle 数据源

- **Gas Price**: Chainstack / Infura / Alchemy RPC `eth_gasPrice`
- **出块时间**: 本地 EWMA 追踪（最近 100 块中位数）
- **桥状态**: Stargate API health + 24h 成功率

## 待定

- [ ] Oracle 多源去重（3 中 2 共识 vs 加权平均）
- [ ] 用户自定义权重（UI 层暴露 speed/cost slider）

## 参考

- axon-design.md §5 路由评分公式
- router.rs prototype sprint
