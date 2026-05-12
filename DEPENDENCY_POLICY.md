# Axon 依赖管理政策

> 创建日期：2026-05-12
> 状态：已生效
> 关联：[HANDOFF.md](HANDOFF.md) / [AXON_PHILOSOPHY.md](AXON_PHILOSOPHY.md)
> 同构：[KubePivot DEPENDENCY_POLICY.md](../KubePivot/DEPENDENCY_POLICY.md)

---

## 哲学根基

**Axon 的依赖管理不是关于"零依赖"，而是关于"依赖不拥有我们"。**

每条外部代码进入项目，必须能回答三个问题：

1. **它侵入核心逻辑吗？** —— 如果它决定了 Axon 如何做 MPC 签名、如何选链、如何管理 Gas，它就不只是"依赖"，而是"共同决策者"
2. **它被限定在边界吗？** —— 如果它只在 I/O 层、可观测性侧支、或序列化层，且失败时不影响主路径，它就可以被隔离管理
3. **我们控制它的退出成本吗？** —— 如果有一天需要替换它，代价是一条 trait 适配器还是整个架构重构？

---

## 三级分级标准

### Level 0 — 禁止：逻辑侵入性依赖

即使有 trait 抽象也无法隔离其对 Axon 核心决策的影响。

**判定标准：**
- 定义了 Axon 会直接调用的核心数据结构或接口，且替换成本涉及架构级重构
- 接管了本应由 Axon 自己实现的执行路径（如 MPC 签名协议、链路由决策）

**当前 Axon 状态：** 无 Level 0 依赖。v0.1.0 的 Cargo.toml 中无禁止项。

**未来候选（需单独论证）：**
- 任何 MPC 签名协议的第三方完整实现（Axon 必须自己实现 GG20 协议核心）
- 任何链上路由决策引擎 SDK（路由评分公式是 Axon 核心 IP）

---

### Level 1 — 受限：标准协议库 / Rust 生态基础设施

允许引入，但必须满足两个条件：(a) 仅用于特定边界层，(b) 失败时零影响核心执行路径。

**判定标准：**
- 实现的是行业通用协议/标准（HTTP/gRPC/序列化），而非 Axon 特有逻辑
- 替换成本中等（需要重写适配层，但不涉及核心架构改动）

**Axon 当前 Level 1 依赖（来自 Cargo.toml）：**

| 依赖 | 用途 | 边界层 | 替换成本 |
|------|------|--------|----------|
| `tokio` | 异步运行时 | 全栈基础 | 高（Rust 生态事实标准） |
| `axum` | HTTP server | 输入/输出层 | 低（trait 切换 tonic） |
| `tonic` / `prost` | gRPC server + protobuf | 输入/输出层 | 低（trait 切换） |
| `serde` / `serde_json` | 序列化 | 输入/输出层 | 低（标准 trait） |
| `tracing` / `tracing-subscriber` | 可观测性 | 侧支 | 低（宏接口） |
| `thiserror` / `anyhow` | 错误处理 | 全栈 | 低（标准 trait） |
| `bytes` | 零拷贝 buffer | 热路径 | 低（标准库 BytesMut） |
| `reqwest` | HTTP client (Oracle) | Oracle 查询 | 低（trait 封装） | ⚠️ 见下方 Axon 独有风险 |
| `rand` | 确定性 seeded RNG | 密码学边界 | 低（替换 rand_core trait） |

**Rust 生态特殊性：**
- `tokio` 是 Rust 异步运行时的事实标准，类似 Go 的 goroutine——不可替换但不属于"侵入核心逻辑"。**但注意**：tokio 从 v0.1.0 起就全栈渗入（main.rs / lib.rs / 所有 async fn），生态耦合极深。历史上 tokio 0.1→0.2→0.3→1.x 的 breaking change 频率高于 Go 标准库。政策：**监控 tokio 版本爆炸**，每次 `cargo update` 后跑全量 `cargo test`，重大版本升级需在决策流程中单独论证。
- `serde` 的 `Serialize`/`Deserialize` trait 是 Rust 生态的标准派生宏，不是框架
- 判定标准不是"替换难度"，而是"它是否修改了 Axon 的核心决策路径"

**Axon 独有风险（KubePivot 无对应）：**
- `reqwest` (L1, Oracle 查询) — KubePivot 用 Go `net/http` 原生标准库做 HTTP 请求，无第三方 HTTP client 依赖。Axon 的 reqwest 在 **Oracle 热路径**上（gas price / bridge health 每 10s 轮询），Oracle 挂了=路由决策盲飞。对策（Sprint 1 落地）：
  1. **tower::retry 层** — 指数退避 3 次重试，防单次网络抖动引发路由降级
  2. **Oracle trait 抽象** — `OracleProvider` trait 注入，测试 mock 谎报数据
  3. **quorum 降级** — 3 个 RPC provider 取 2/3，单个 reqwest 超时不阻塞全局

---

### Level 2 — 允许：零侵入 CLI wrapper

通过进程调用外部二进制，Axon 不链接任何外部 SDK。失败时优雅降级。

**判定标准：**
- 外部工具独立运行，通过结构化输出（JSON）或标准协议（HTTP/gRPC）与 Axon 交互
- 失败时 Axon 优雅降级（fallback 路径或明确错误提示）

**当前 Axon Level 2 依赖：**

| 工具 | 用途 | 调用方式 |
|------|------|----------|
| kubectl | K8s 资源操作 | CLI (kp deploy 链路) |
| helm | Helm chart 操作 | CLI (kp deploy 链路) |
| docker | 镜像构建 | CLI (Makefile) |

**未来候选（需在实施前判定）：**
- etcd — 状态存储（tonic gRPC client 直连，非 CLI wrapper，按 Level 1 判定）。**警戒线**：etcd 当前仅存 idempotency key / nonce TTL / audit trail——纯 KV 状态层，L1 安全。但若未来 etcd 中的路由状态（如 "which chain is preferred for this user"）开始**影响 Axon 的路由决策路径**，etcd client 从 L1 直升 L0。到那时需论证：是用 etcd 的 watch API 做事件分发（仍可 L1），还是 etcd 数据直接决定 `router.route()` 的返回值（L0 红线）。
- Stargate / CBridge API — 桥接协议（HTTP JSON，按 Level 1 边界层判定）

**Tier C 热路径新增候选：**

| 候选 | 用途 | 判定 | 触发条件 |
|------|------|------|----------|
| `bumpalo` | arena alloc（签名树 / 多 obj 同生命周期） | L1 dev-dependency 仅 bench/fuzz 用 | 热路径 alloc > 10/transfer 且 flamegraph 证实在 bump 场景 |
| `flamegraph` | CPU 火焰图 | L2 CLI wrapper (`cargo flamegraph`) | Tier B alloc 阈值告警时启用 |
| `valgrind --tool=massif` | 堆内存峰值分析 | L2 CLI wrapper（系统二进制） | 内存泄漏排查 / RSS 异常时启用 |

---

## 决策流程

新依赖进入 Axon 时的判定路径：

```
1. 它能通过 CLI wrapper 模式使用吗？
   → 是：Level 2 — 零侵入允许
   → 否：进入问题 2

2. 它是 Rust 生态基础设施 / 行业通用协议实现吗？
   → 是，且仅用于边界层（I/O/序列化/可观测性）：
     进入 Level 1 判定。论证"失败不影响核心路径"的具体机制
   → 否，或虽然通用但侵入核心逻辑：进入问题 3

3. 它会接管 Axon 的核心执行路径吗？
   → 是：Level 0 — 禁止。自研替代或拒绝
   → 否：回到 Level 1 判定
```

---

## 重新评估

此政策在以下时机重新评估：
- 每个 Sprint 开始时：检查 Cargo.toml 新增依赖
- 引入新的密码学库（MPC 签名）时：确保不引入完整 MPC 协议实现
- 引入新的链 SDK（如 ethers-rs）时：确保不接管路由决策
- 任何 `cargo add` 引入不在当前清单中的 crate 时

---

## KubePivot 同构

```
KubePivot (Go)                  Axon (Rust)
─────────────────────────────────────────────
不引入 client-go                不引入完整 MPC 协议实现
不引入 Casbin/OPA               不引入第三方路由引擎
prometheus/client_golang L1    tracing L1（都是可观测性侧支）
kubectl/helm L2 CLI wrapper    kubectl/helm L2（同一工具链）
execCommandFunc mock 注入      trait + Box<dyn Fn> 注入（见 MISTAKES.md R04）
```
