# ADR 005 — 跨链桥安全

> 日期：2026-05-11 | 状态：draft | 作者：qc + DeepSeek

---

## 决策

**主桥 Stargate (LayerZero)**，备选 CBridge。白名单制，不动态发现。Axon 不上链桥路由，仅作 off-chain 决策 + 参数注入。

## 为什么 Stargate

| 维度 | Stargate | Wormhole | LayerZero 原生 |
|------|----------|----------|---------------|
| TVL | $500M+ | $1B+ | N/A (infra) |
| 桥梁模型 | 统一流动性池 | Guardian 多签 | 超轻节点 (DVN) |
| 链覆盖 | 10+ EVM | 30+ (含 Solana) | ~50 (需自定义 DVN) |
| 审计 | Zellic + Trail of Bits | Kudelski + Neodyme | Zellic |
| 既往事故 | 0 | 1 ($326M, 2022) | 0 |

Stargate 选型：**统一流动性池** = 跨链资产不需要对端有对应 LP 池。LayerZero DVN 架构更分布式（vs Wormhole 19 Guardian），且无历史事故。

## 白名单

```
主:    Stargate (stargate.finance)
备:    CBridge (cbridge.celer.network)
禁:    未审计桥 / TVL<$50M / Bridge hacks 24个月内
```

白名单在 `configs/system.yaml` 维护，不依赖动态链上发现。

## 责任边界（写死）

```
Axon 路由/配置错误       → Axon 全额兜底
桥故障/链拥堵/回滚/黑客  → 按桥方规则, Axon 协助追查
用户填错地址/链          → 用户承担, Axon 尽力协助
```

标准文案：
> "跨链基于第三方桥协议，因桥服务故障导致的异常，平台协助追踪但不承担资金兜底责任；因平台路由配置错误导致的失败，由平台全额兜底并重新发起。"

## 集成模式

```
Axon Router (off-chain)
  │
  ├─ score → selected chain
  ├─ bridge check → Stargate whitelist pass
  │
  ▼
web3-blitz BridgeAdapter
  ├─ StargateRouter.sol   (链上桥调用)
  ├─ BridgeFee 预估 (Stargate API)
  └─ 状态追踪 (src tx hash → dst tx hash)
```

Axon 只做 off-chain 评分 + bridge 选择，链上执行由 `web3-blitz` 的 BridgeAdapter 负责。不重复造桥。

## 安全措施

1. **单笔上限**: bridge tx value < $100k 等值（超过需人工审批）
2. **桥状态监控**: Stargate API health check 每 30s，连续 3 次 5xx → 自动切 CBridge
3. **黑洞地址**: 目的地址校验（非 0x0 / 0xdead / 已知黑洞）

## 待定

- [ ] 桥费用缓存 TTL（避免每次跨链前 API 调用）
- [ ] 自动切桥的恢复阈值（Stargate 恢复后多长时间切回）
- [ ] 跨链 tx 追踪 UI (src hash → dst hash link)

## 参考

- axon-design.md §5.3 跨链转账处理
- [Stargate Docs](https://stargateprotocol.gitbook.io/stargate)
- [LayerZero Integration](https://layerzero.gitbook.io/docs)
