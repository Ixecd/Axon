# FORGET.md — 待修复项（P0 + P1）

> 扫描日期：2026-05-12
> 范围：`docs/design/` 下 5 个 ADR + `src/` 代码骨架
> 原则：只列 P0（生产命门）和 P1（功能受限），P2 Ops / P3 Polish / 长期演进不提

---

## P0 — 生产命门（上生产前必修）— 0/4

### MPC / 签名

1. **MPC GG20 状态机未实现** — `src/mpc.rs` 尚不存在，FIXME: Sprint 2。依赖：tokio::Mutex 锁模式 + `enum StateMachine` + `mem::replace`（见 MISTAKES.md R01）。

2. **MPC dropout recovery 未实现** — t-of-n 中 n-t 个节点超时不阻塞，异步 catch-up。ADR 001 已设计，代码零行。

### Gas / Paymaster

3. **Paymaster 完全未动** — `src/paymaster.rs` 不存在。gas vault / pre-est / jitter buffer 均为零。ADR 004 已设计，Sprint 2 待办。

### Bridge

4. **Bridge finality watch 未实现** — `src/bridge.rs` 不存在。Stargate finality poll / CBridge fallback 均为零。ADR 005 已设计。

---

## P1 — 功能受限（规模化前必做）— 0/5

### Router

5. **Router 多链评分未实现** — `src/router.rs` 仅空 struct。Oracle quorum（3中2）/ route scoring 公式 / bridge whitelist 零行。ADR 003 已设计，Sprint 1 当前。

6. **RPC quorum 未接线** — 3 独立 RPC provider 取 2/3 一致逻辑未写。依赖 Router 实现。

### AA / 账户

7. **ERC-4337 AccountFactory + Paymaster 未实现** — ADR 002 已设计，Session key / social recovery 均为零。依赖 Paymaster 模块。

### 部署

8. **chaos-mesh 验收未接通** — AXON_PHILOSOPHY 要求 "chaos-mesh kill mpc pod → sig 恢复不丢不重" 才算 S+ 过。需 MPC 实现后才可接入。

### 数据模型

9. **etcd schema 未定义** — transfer history / idempotency key / nonce TTL 均需 etcd 存储，schema 零设计。

---

## 编辑记录

```
2026-05-12  v0.1 初始扫描
            - 5 ADR + src/ 骨架全扫
            - P0 4 项：MPC 状态机 / MPC dropout / Paymaster / Bridge
            - P1 5 项：Router 评分 / RPC quorum / AA / chaos-mesh / etcd schema
            - 全部 open，符合 v0.1 早期阶段预期
```
