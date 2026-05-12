# Axon ROADMAP

> 创建日期：2026-05-12
> 当前版本：v0.1.0
> 原则：确定性工程交付。模糊想法 → [FUTURE.md](FUTURE.md)

---

## v0.1.0 — Rust scaffold（当前）

### 已交付

- `src/main.rs` — axum HTTP server，`/healthz` + `/` 两个路由
- `src/router.rs` — Router struct 骨架（Sprint 1 原型）
- `Cargo.toml` — axum + tonic 双栈依赖，release LTO/panic=abort
- `docs/design/` — ADR 001-005（MPC / AA / Router / Gas / Bridge）
- `deployments/` — Helm charts（axon / axon-etcd / axon-postgres）
- `configs/` — KubePivot 部署配置（components.yaml / system.yaml / resources.yaml）
- `build/docker/axon/` — Dockerfile（rust:1.85-alpine → scratch，~40MB）
- `AXON_PHILOSOPHY.md` — 开发哲学 & 规范指南
- `MISTAKES.md` — 重复性错误日志
- `FORGET.md` — P0+P1 待修复清单
- `DEPENDENCY_POLICY.md` — 依赖管理三级分级
- `FUTURE.md` — 架构种子库

---

## v0.2 — Sprint 1: Router + 单链 tx

### 目标

```
router 多链评分 + RPC quorum + 单链 tx 跑通
→ kp deploy → 注入脏（RPC mock 谎报 / Gas spike 模拟）
```

### 核心交付

- Router 多链评分引擎（Speed × 0.6 + Cost × 0.3 + Safety × 0.1）
- Oracle quorum：3 RPC provider，2/3 共识
- Bridge whitelist：Stargate 主，CBridge fallback
- 单链 transfer 端到端（mock MPC 签名）
- RPC dirty fuzz 测试（mock 谎报数据 → router 检测 fallback）

### 验收

```
✓ Router score <5ms bench
✓ RPC quorum 3中2 通过
✓ 单链 tx 在 kp deploy 环境跑通
✓ mock RPC 谎报 → router 检测并 fallback
✓ cargo test --lib 全绿
✓ cargo clippy -- -D warnings 零报错
```

---

## v0.3 — Sprint 2: MPC + gRPC

### 目标

```
MPC GG20 真实签名 → tonic gRPC server → Paymaster gas 代付
```

### 核心交付

- MPC GG20 t-of-n 状态机（Idle → Init → Signing → Done）
- MPC dropout recovery（t-of-n 中 n-t 节点超时不阻塞）
- Paymaster pre-est + gas vault + jitter buffer
- etcd state store 接线（idempotency key / nonce TTL）
- chaos-mesh kill mpc pod → sig 恢复不丢不重

### 验收

```
✓ MPC 签名 <200ms p99
✓ MPC pod kill → 签名恢复不丢不重（chaos-mesh）
✓ POST /api/v1/transfer 端到端（含 gas 代付）
✓ etcd idempotency key 防重放
```

---

## v0.4 — Sprint 3: 跨链 Bridge + 前端壳

### 目标

```
跨链 transfer → Tauri app shell → 4 统一 API 对外
```

### 核心交付

- Bridge finality watch（Stargate 10s poll / CBridge fallback）
- 跨链 transfer 端到端（Ethereum ↔ Polygon）
- 4 统一 API：balance / deposit / transfer / swap
- Tauri v2 app shell（React + tonic client 直连）
- 设计系统落地（暗底策展 + 暖调画廊双模）

### 验收

```
✓ 跨链 transfer <30s finality 检测
✓ Bridge stuck 30min → auto CBridge fallback
✓ 4 API 全部可用（HTTP + gRPC）
✓ Tauri app 5MB 内
```

---

## 平台线 & 壳线

```
平台线: POST /transfer — MPC<200ms, Gas代付, 跨链 Stargate
壳线:   炒币 view (CLOB UI), self-custody order, AA batch sig, 平台不撮

不做的:
  ✗ 撮合引擎 (链上 DEX 已有)
  ✗ 清结算系统 (链本身就是)
  ✗ CEX 仓位管理 (绝不)
```

---

## 版本号规则

- 主版本号 0：MVP 阶段，一切可变
- 次版本号递增：每个 Sprint 完成打 tag
- 不打 patch 版本

```
v0.1.0 → v0.2.0 (Sprint 1) → v0.3.0 (Sprint 2) → v0.4.0 (Sprint 3)
```

---

## 关键依赖链

```
Router (v0.2) → RPC quorum → Oracle 数据流
MPC (v0.3)    → etcd state store → Paymaster → idempotency
Bridge (v0.4) → Stargate API → CBridge fallback → finality watch
```

---

## KubePivot 同构

```
Axon Sprint           KubePivot 同构
────────────────────────────────────────
Router 多链决策       Scheduler 多维 DP + 评分
MPC 状态机            状态机 (12 deploy state + 5 sandbox)
Paymaster Gas 代付    WorkerPool token bucket 背压
RPC quorum            InformerDetector 双保险
Bridge fallback       Rescheduler 多级降级
chaos-mesh 验收       chaos-mesh kill controller pod
```
