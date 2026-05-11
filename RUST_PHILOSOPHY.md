# Axon Rust 开发哲学 & 规范指南 😈✊

> 作者：庆春镜像 + DeepSeek | 日期：2026-05-11 | 版本：v2 锐评修正版
> 核心：**Correctness First • 业务脏=燃料 • 金融压制** — MPC+AA 非托管核，Rust 铁律执行，像微信但链上杀手。
> 适用：Axon Protocol (router/mpc/paymaster/bridge)，Rust 1.80+。
> 原则：**零自欺** — 代码=产品，债不堆，数字非纳秒迷信是可测边界。

---

## 一、Tier 铁锁（S+ → A → B 优先链）

优先级不是线性的是降级门槛制：**S+ 不过关不可上线，A 不达标不可规模化，B 是 debt metric 不是信仰。**

### S+ — 正确性（生产命门，P0=0 才 merge）

| 能力 | Axon 需求 | KubePivot 同构 |
|------|----------|---------------|
| Deterministic replay | 相同 tx → 相同 sig，router 回放可审计 | FSM 状态机回放 |
| Audit trail | 每笔 transfer 全路径可溯源 | etcd watch log |
| Rollback safety | Paymaster refund 原子性，Gas 无泄漏 | state.StateMachine |
| MPC sig verify | t-of-n 签名验证 + dropout recovery | sharding lease 续约 |
| Dirty RPC fuzz | 多节点 quorum（3中2）+ rpc 返回伪造数据注入测试 | InformerDetector 双保险 |

**判定标准**：proptest fuzz `router` / `mpc_sign` 输入空间，`cargo-fuzz` 跑 24h 零 crash。CI 绿 ≠ S+ 过，**chaos-mesh kill mpc pod → sig 恢复不丢不重** 才算。

### A — 延迟（规模化门槛）

| 指标 | 目标 | 测量方式 |
|------|------|---------|
| 完整 MPC 签名 | <200ms (GG20 4-6轮) | `criterion` bench p99 |
| Router 决策 | <5ms (10链评分) | `criterion` bench |
| Bridge finality poll | <30s 检测 stuck | Stargate API health watch |
| Paymaster pre-est | <50ms (Oracle + Gas) | reqwest timeout gated |

对齐：KubePivot Rescheduler <30s 扫描 → Axon bridge finality <30s。

### B — 资源效率（debt metric，非信仰）

```
hotpath alloc: <10 alloc/transfer — bytes::BytesMut (自带内部池化)
                超阈值 = CI warn，不 block merge，但累积 > N 周 → 自动升 A

binpack:      <500ms (10 nodes, 100 pods) — 非热路径，业务后优化
tracing span: <1μs overhead

戳宗教: 数字不是纳秒迷信，是可测边界。
         局部 opt 杀演化 > 10μs 收益。
         benchmark 是尺子不是神像。
```

---

## 二、业务脏 = 起飞燃料（核心章）

**"业务不脏"是幻觉。脏才是金融系统的真实世界。代码能扛脏才是真正确性。**

### 2.1 半夜错链

```
场景: 用户选 Polygon，Oracle 延迟返回 Mainnet Gas → Router 误路由 Mainnet
代价: 用户付 $50 Gas vs 预期 $0.5

解法:
  1. Router replay: tx 提交前 2s 内二次确认 oracle 数据 fresh
  2. User confirm: AA batch 里嵌入预计费用，用户 session key 签批
  3. 异常回退: Gas 实际消耗 > 预估×1.5 → Paymaster 拒绝，走 user-pay fallback

KubePivot 同构: InformerDetector cache hit → kubectl fallback 二次验证
```

### 2.2 Bridge 卡死

```
场景: Stargate 桥 tx 提交后 30min 不到账，dst 链无 tx hash
代价: 用户资金悬空，焦虑工单

解法:
  1. Stargate finality watch: 每 10s poll src/dst tx status
  2. 超 30min → 自动 fallback CBridge（如果资产在 CBridge 有 LP）
  3. 无 fallback → 工单 + tx proof 直接推 Stargate 客服
  4. Axon 不兜底资金（但协助追踪）

KubePivot 同构: Rescheduler 抖动降级 Level 0→1→2→3
```

### 2.3 Gas 不够

```
场景: 提交时 Gas=50gwei，广播时 spike 到 300gwei → tx pending 不确认
代价: 用户资金锁定，MPC 签名已消耗但 tx 失败

解法:
  1. Paymaster pre-est: 提交前 1s 内取 maxFeePerGas × 安全系数 1.5
  2. Oracle jitter buffer: EWMA 追踪最近 10 block 的 gas 波动
  3. 超 buffer → 拒绝垫付 + 通知用户（不等 pending timeout）
  4. 已提交未确认 → 同 nonce 加速 tx (replace-by-fee)

KubePivot 同构: WorkerPool token bucket backpressure
```

### 2.4 RPC 撒谎

```
场景: RPC 节点返回假 balance / 假 receipt / 过期 block
代价: 路由选错链 / 误判 tx 成功 / 双花风险

解法:
  1. Multi-node quorum: 3 个独立 RPC provider 取 2/3 一致
  2. Sig verify: 收到 receipt 后本地验证签名（不信任 RPC 返回的 status）
  3. Block freshness: block.timestamp 偏移 > 30s → 拒绝
  4. 注入测试: test 里 mock RPC 故意返回错数据 → router 必须检测并 fallback

KubePivot 同构: etcd corrupt-check 每 10min 巡检
```

### 2.5 重放 / 超时

```
场景: 用户点击两次 → 同 tx 提交两次（nonce 相同但内容不同）
      MPC 网络延迟 → 部分分片超时 → 签名协议中断

解法:
  1. Nonce + TTL: 每 tx 带 monotonic nonce + 30s expiry
  2. MPC dropout recovery: t-of-n 中 n-t 个节点超时不阻塞
     比如此轮 2/3 在线 → 推进签名（1 个节点异步 catch-up）
  3. Idempotency key: tx hash 写入 etcd，重复提交返回已有结果

KubePivot 同构: state.StateMachine idempotent reconcile
```

---

## 三、升维推 — 双帝炼狱

```
KubePivot = cloud runtime  (容器调度 / 自愈 / 扩容)
Axon       = financial runtime (链上交易 / MPC 签名 / Gas 代付)

同构骨架:
  状态机     KubePivot deploy FSM (12 state)  → Axon MPC FSM (Idle→Init→Signing→Done)
  分片       KubePivot sharding Lease          → Axon MPC key shard lease
  背压       KubePivot token bucket            → Axon Paymaster quota
  双保险     KubePivot InformerDetector        → Axon RPC quorum
  降级链     KubePivot Rescheduler Level 0-3   → Axon bridge fallback + gas jitter

Sprint 1 策略:
  router + 单链 tx 跑通 → 注入脏（RPC mock 谎报 / Gas spike 模拟）
  → kp deploy → chaos-mesh kill mpc pod → metrics audit
  脏锤铁链，从第一天就扛真实世界。
```

---

## 四、Go→Rust 过渡坑

```
Go 自然写法              Rust 等价 / 坑
─────────────────────────────────────────────────────
smLock.Lock()            let guard = self.sm.lock().await;
defer smLock.Unlock()    // tokio::Mutex guard 不能跨 await 点持有
                          // std::Mutex guard 不能跨 .await
                          解: enum StateMachine + mem::replace

fmt.Errorf(...)          anyhow::bail!(...) 或 anyhow::anyhow!(...)
                         坑: bail! 需要 use anyhow::bail;

goroutine + channel      tokio::spawn + tokio::select!
                         坑: spawned task 的 JoinHandle 必须 await 或 abort
                         否则 task panic 被静默吞

var newFunc = realImpl   函数变量注入 (KubePivot 模式)
test: newFunc = fake     Rust: trait + Box<dyn Fn> 注入，不用全局 var
```

---

## 五、模块规范

```
src/
├── lib.rs          # pub mod {router,mpc,paymaster,bridge};
├── router.rs       # 多链决策 + RPC quorum
├── mpc.rs          # 协同签名状态机 + dropout recovery
├── paymaster.rs    # Gas vault + pre-est + jitter buffer
├── bridge.rs       # Stargate 接入 + finality watch
├── error.rs        # thiserror enum
└── main.rs         # tokio::main tonic server
```

---

## 六、代码规范（核心片段）

### Error

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AxonError {
    #[error("MPC shard missing: {shard_id}")]
    MpcMissing { shard_id: u32 },
    #[error("Gas exceed: {actual} > {limit}")]
    GasExceed { actual: u64, limit: u64 },
    #[error("Invalid state transition")]
    InvalidState,
}

pub type Result<T> = std::result::Result<T, AxonError>;
```

### MPC 状态机

```rust
use anyhow::bail;

#[derive(Debug)]
enum MpcState {
    Idle,
    Init { shards: Vec<u32> },
    Signing { ctx: SigCtx },
    Done(Result<Sig, AxonError>),
}

impl Mpc {
    async fn next(&mut self) -> Result<()> {
        let state = std::mem::replace(&mut self.state, MpcState::Idle);
        match state {
            MpcState::Idle => bail!(AxonError::InvalidState),
            MpcState::Init { shards } => {
                self.state = MpcState::Signing { ctx: SigCtx::new(&shards)? };
            }
            MpcState::Signing { ctx } => {
                let sig = ctx.execute_round().await?;
                self.state = MpcState::Done(Ok(sig));
            }
            MpcState::Done(_) => {}
        }
        Ok(())
    }
}
```

### Router（Oracle quorum）

```rust
impl Router {
    /// 3 RPC providers → 取 2/3 quorum → 评分
    pub async fn route(&self, tx: &Tx) -> Result<ChainId> {
        let oracles = self.oracles.quorum_gas_price(tx.chain_hint).await?;
        self.chains.iter()
            .filter(|c| self.bridge_whitelist.contains(&c.chain_id))
            .map(|c| (c.score(tx, &oracles), c.chain_id))
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(_, id)| id)
            .ok_or(AxonError::NoRoute)
    }
}
```

---

## 七、Sprint 1 铁律

```
目标: router + 单链 tx 跑通 (axum HTTP → 后续切 tonic)
时间: 1 天 prototype
验收:
  [ ] Router score 10 链 <5ms (criterion bench)
  [ ] RPC quorum 3中2 检测 RPC 异常返回
  [ ] Mock Gas spike → Paymaster pre-est 拒绝
  [ ] kp deploy → chaos-mesh kill pod → metrics 不丢
  [ ] cargo-fuzz router input 1h 零 crash

不做的:
  ✗ MPC sig（sprint 2）
  ✗ Bridge Stargate 真连接（mock 先）
  ✗ AA 合约部署（testnet 后）
```

---

## 八、命令

```bash
cargo clippy --fix --all-targets -- -D warnings
cargo fmt
cargo test --lib
cargo criterion          # bench
cargo fuzz run router    # 24h before release
cargo audit              # weekly
cargo deny check         # license + crypto whitelist
```

## 九、FORGET 节奏

每周 `cargo audit` + `cargo deny` + grep `TODO|FIXME|unsafe`。P0=0 才 merge。
P0 = S+ 正确性项，P1 = A 延迟项。B 是 debt metric 不设 P-level。

**结语**：Rust=压制力，KubePivot=cloud runtime，Axon=financial runtime。双帝炼狱，脏锤铁链，金融压制⛓️😈✊。
