# FUTURE.md — Axon 架构演化实验室

> 创建日期：2026-05-12
> 作用：存放高价值、高风险、需要"重新评估时机"的架构种子
> 关联：[ROADMAP.md](ROADMAP.md)（确定性工程交付计划）
> 同构：[KubePivot FUTURE.md](../KubePivot/FUTURE.md)

---

## 这份文档是什么 / 不是什么

`ROADMAP.md` 是 **军令状** — 写的是未来 Sprint 内要交付的确定性工程。
`FUTURE.md` 是 **实验室** — 存放需要在合适时机重新评估的远景种子。

**这份文档不是承诺**：里面的每一条都不保证会实施。
**这份文档不是 backlog**：不是"想做但没时间"的待办池。
**这份文档是种子库**：当条件成熟时（用户规模 / 链生态成熟度 / 工程容量），重新评估、决定是否提升到 ROADMAP。

每个种子必须包含：
- **动机**：为什么这个想法值得记录
- **核心想法**：一段话讲清楚是什么
- **工程税**：诚实列出实施成本和风险
- **适用前提**：什么条件下重新评估才有意义
- **重新评估时机**：与 ROADMAP 的交点

---

## 种子库

| 编号 | 种子 | 状态 | 重新评估时机 |
|---|---|---|---|
| F1 | ZK-proof 审计追踪 | 🌱 探索中 | v0.3+ MPC 签名稳定后 |
| F2 | Intent-based 交易（ERC-7683） | 🌱 探索中 | v0.4+ 跨链路由成熟后 |

（种子库应该谨慎扩张。空表不是问题，比乱填种子好。）

---

## F1: ZK-proof 审计追踪

> 每笔 transfer 附带零知识证明，实现"可验证但不可见"的审计。第三方可验证交易合法性而不暴露金额/地址。

### 动机

Axon 的核心承诺是"self-custody，平台不持不撮"。但如何向用户和监管证明这一点？

当前方案（ADR 001-005）依赖：
- etcd 全链路 audit trail
- MPC t-of-n 签名（平台从不持完整私钥）

但 audit trail 存在 etcd 中→平台可以改。**技术上平台可以作恶**，只是我们承诺不作恶。

ZK-proof 提供更强的保证：
- 用户钱包本地生成 proof：证明"此交易由我签名，金额<我的余额"
- Axon 提交 proof 上链，任何人都可验证
- 不暴露 from/to/amount，但数学上可证明交易合法

### 核心想法

```
当前 audit trail:
  平台存 etcd → 平台承诺没改 → 用户信任平台

ZK audit trail:
  用户本地生成 proof → 上链存证 → 任何人零知识验证
  平台改不了链上数据，平台作恶成本=共识攻击成本
```

### 工程税

1. **密码学复杂度**：Groth16 / Plonk 证明系统 + 电路设计（transfer 约束电路）
2. **用户设备性能**：移动端生成 proof（~2-5s on phone）→ 可用性挑战
3. **链上 gas 成本**：proof 验证 ≈ 200k gas on L1 → 需 L2 或批量验证
4. **电路维护**：每次 transfer 逻辑变更→电路变更→重新 trusted setup
5. **团队密码学能力**：当前团队无 ZK 密码学背景→需外部审计

### 适用前提

- [ ] MPC 签名 + transfer 核心稳定（至少 v0.3）
- [ ] 用户规模 ≥ 1000，有审计/合规需求
- [ ] 团队有 ZK 密码学能力或外部审计资源
- [ ] L2 生态成熟到 proof 验证 gas 可接受（< $0.01/verification）

### 重新评估时机

**v0.4 设计阶段**（约 2026-Q4）重新评估。

如果决定不做：
- 保留 etcd audit trail + 第三方审计接入
- 公开 MPC signer 身份（reputational staking）作为折中方案

如果决定做：
- 提升到 ROADMAP.md
- 先做 PoC：单笔 transfer 的 Groth16 proof 原型

---

## F2: Intent-based 交易（ERC-7683）

> 用户表达意图（"我想用 100 USDC 换 ETH"），Solver 竞价执行最优路径，Axon 从 Router 升级为 Intent Clearinghouse。

### 动机

Axon 当前架构是 **Router 集中决策**：Router 计算最优链+桥，用户被动接受。

Intent-based 模型翻转这个关系：
- 用户签名 intent（"用 100 USDC 换至少 0.03 ETH，5 分钟内"）
- N 个 Solver 竞价抢单，出价最高的赢
- Axon 从"做决策的人"变成"保证规则公平的平台"

这是 UniswapX / CoWSwap 的模型，但 Axon 可以做多链版本。

### 核心想法

```
当前 Router 模型:
  用户 → Router 选链 → Bridge → DEX → 完成
  问题: Router 是最优吗？谁验证 Router？

Intent 模型:
  用户 → 签名 intent → N Solver 竞价 → 最优者执行
  Axon 角色: 验证 intent 合法性 + 托管结算（MPC 释放资金）
```

### 工程税

1. **Solver 网络冷启动**：没有 Solver=没有流动性=intent 无人接
2. **MEV 保护**：Solver 可能 front-run 用户 intent，需要批量拍卖机制
3. **跨链 intent 结算**：ERC-7683 标准仍在草案阶段，跨链 fill 复杂
4. **与现有 Router 的共存**：不能废弃已有 Router，需要双模式

### 适用前提

- [ ] Router 单链路由稳定运行（至少 v0.3）
- [ ] ERC-7683 标准正式发布
- [ ] 至少 3 个 Solver 愿意接入（冷启动伙伴）
- [ ] 跨链 bridge finality watch 已稳定

### 重新评估时机

**v0.5+ 设计阶段** 重新评估。ERC-7683 标准仍需观望。

---

## 种子状态流转

```
🌱 探索中 → ✅ 已毕业（提升到 ROADMAP）
         → ❌ 已弃用（评估后确定不做，保留决策记录）
         → ⏸️ 长期搁置（条件不成熟）
```

种子被弃用时**不删除**，保留作为"历史决策依据"。

---

## 编辑记录

```
2026-05-12  创建文档 + F1 (ZK-proof audit) + F2 (Intent-based/ERC-7683)
```
