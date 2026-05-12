# Axon Rust 开发哲学 & 规范指南

> 作者：庆春镜像 + DeepSeek | 日期：2026-05-13 | 版本：v3 Payroll 修正版
> 核心：**薪酬支付 • 身份绑定 • 到账止步** — MPC+AA 非托管核，Rust 铁律执行。
> 适用：Axon Protocol (identity/router/mpc/paymaster/bridge)，Rust 1.80+。
> 原则：**零自欺** — 代码=产品，债不堆，数字非纳秒迷信是可测边界。

---

## 零、Axon 是什么 — Feelings 链上薪酬支付引擎

**核心永不动摇：发工资。** Axon 是 Feelings 项目的内部薪酬支付层。`disburse(treasury, empId, amount)` — 把工资从 Feelings 财务地址发到员工/创作者的钱包地址。

```
用户看到的              Axon 实际做的
─────────────────────────────────────────
员工领工资            treasury → resolve(empId) → transfer (MPC sig)
创作者分润            收益上链 → 自动结算 → 创作者地址
跨链发放            Polygon treasury → Bridge → 员工 Arbitrum 钱包
```

平台不做的事：
- **不撮合** — 没有 DEX，没有 order book，没有 trading UI
- **不跟踪消费** — 工资到账即 Axon 职责结束，员工怎么花无关
- **不持仓** — 零 CEX 仓位风险，资金从 treasury 直达员工地址

核心边界：**"到账止步。"** 发工资是 Axon 的事，消费是员工自己的事。

架构锚定：

```
Axon Server (Rust tonic gRPC)
├── Identity Registry   — empId → 地址绑定 (MVP: empId+合同哈希)
├── Router              — 最优链选择 (Gas cheapest)
├── MPC Signer          — 企业级签名
├── Paymaster           — Gas 代付 (员工零 Gas)
└── Bridge              — 跨链 disbursement
```

**身份层**：`POST /bind` — empId + 实名 + 合同哈希 → 钱包地址，存 Registry。
**发薪核**：`POST /disburse` — resolve(empId) → route → MPC sign → broadcast。到账即止。

风控 bounded（企业级 payroll，非交易所）：

| 层 | 风控 | 组件 |
|----|------|------|
| 身份层 | empId 解析失败 → 拒；非活跃员工 → 拒 | EmployeeRegistry |
| 发薪核 | 异常 vol/velo/重放 | JRaft 状态 + Aeron alert |
| 清结算 | Gas vault + bridge finality | Oracle 对冲 + multi-sig |

铁文案：**"发薪到账，管发不管花。"**

---

## 一、Tier 铁锁（S+ → A → B → C → D 五层锁）

优先级不是线性的是降级门槛制：**S+ 不过关不可上线，A 不达标不可规模化，B 是 debt metric 不是信仰，C 是热路径编码铁律，D 是 Token 预算红线。**

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

### C — 热路径设计原则（业务代码准则）

Axon 热路径 = transfer/sec（目标 >1k TPS）。原则：**零 alloc / 栈优先 / 懒至终**。infra 转业务核心：别 premature abstract，先生成脏代码跑通，再 refactor。

#### C1. 惰性求值 (Lazy First)

```
何时 lazy:
  iter/filter/map 链 > 3 步 → 不 collect，直到下游需要 owned。
  阈值: lazy 链 < 10μs CPU 时间 → 不 collect。

何时 eager:
  需要 len() / index / 多次迭代 / 跨线程 send → 才 collect。
  例: chain config 一次性 load 后长期复用 → OnceCell<Vec<Chain>> 是合理 eager。
```

```rust
// 坏: premature collect — 分配一个 Vec 只为取 max
let scores: Vec<_> = chains.iter().map(score).collect();
let best = scores.into_iter().max_by(cmp_score);

// 好: lazy fuse — 计算沿链走，零 alloc
let best = chains.iter().filter_map(score).max_by(cmp_score)?;
```

```
懒初始化:
  Chain config  → OnceCell<ChainRegistry> 懒 init，不定死在 new()
  静态配置     → LazyLock / OnceLock（Rust 1.80+ std）
  运行环境     → configs/system.yaml → 首次访问时解析，不预加载全局
```

#### C2. 链式调用 (Fluent Pipeline)

```
全 Req 处理:
  Req::validate()?.route()?.sign()?.broadcast()

Builder only for config:
  TransferBuilder::new().chain("eth").gas(21e3).signer(key).build()?

零 state pipe:
  trait Pipeline { type In; type Out; async fn process(&self, input: Self::In) -> Result<Self::Out>; }
  impl Pipeline for Transfer { ... }  // stateless，纯 ? 链

禁:
  ✗ 嵌套 if-else 金字塔
  ✗ unwrap() / expect() 在生产路径
  ✓ 纯 ? 链 + map_err / and_then
```

#### C3. 高效内存 (Stack > Heap > Pool)

```
热路径零 alloc: <10 alloc/transfer（对齐 Tier B）

策略         用法               示例
────────────────────────────────────────────
Stack-only   小 struct <128B    #[repr(C)] struct SigCtx { buf: [u8; 64]; }
Cow<'a>      借用优先           let payload = Cow::Borrowed(&req.data);
BytesMut     热路径 buffer      let mut buf = BytesMut::with_capacity(1024);
Arena        多对象同生命周期    bumpalo::Bump for 签名树
```

```
Cache line 对齐:
  #[repr(align(64))] struct HotPath { ... }
  热字段聚拢，冷字段 #[repr(C)] 末尾

测:
  cargo flamegraph — 目标 <1% alloc time in hotpath
  valgrind --tool=massif — 峰值内存曲线
```

#### C4. 必要抽象 (Rule of Three)

```
何时 abstract:
  3+ 处类似代码 → trait
  1-2 处重复 → 留着，观测

泛型 vs dyn:
  ┌──────────┬─────────────────────┬──────────────────┐
  │          │ 泛型 (static dispatch)│ dyn Trait        │
  ├──────────┼─────────────────────┼──────────────────┤
  │ 分发     │ 零 vtable，编译时单态化 │ +8ns vtable 跳转 │
  │ 适用     │ 调用方 ≤ 16 种       │ 多态 / mock 注入  │
  │ 二进制   │ 每实例膨胀约 5KB     │ 不膨胀           │
  │ Axon     │ RouteScore<T: Chain> │ OracleProvider    │
  └──────────┴─────────────────────┴──────────────────┘

抽象税:
  每 trait +5% 编译时间，+10% 认知成本
  测试覆盖 > 90% 才允许引入新 trait
  每个 trait 必须在 PR 描述列出"三个具体受益者"

业务 code 节奏:
  先 concrete funcs（3 个文件内），refactor 时 abstract
  反模式: 第一版就画 trait 图 → 不写代码先画框
```

**审计流程**: PR 必跑 `cargo criterion --bench hotpath`，diff > 5% reject。

---

### D — Infra Token 节流（迭代零和游戏）

Token 预算是有限的。每轮 YAML 缩进调试 = 3-5x Token 乘数。Infra 不该吃掉业务思考的 Token 配额。

**刺客榜 (Axon 体感)**:

| 刺客 | Token 杀招 | 解药 | 节省 |
|------|-----------|------|------|
| YAML/Infra | 缩进敏感+全读重改循环 | `kp deploy --dry-run=client` + kustomize | -90% (CLI替10轮) |
| Debug日志 | `kubectl logs` 噪音洪水 | `tracing::span!` JSON + `jq '.level=="error"'` | -70% (结构filter) |
| Schema重复 | proto→Rust→TS 五遍 | `prost-build` + `schemars` auto-gen | -60% (一键派生) |
| Test Fixture | mock state 膨胀 | `proptest` + fixture bin | -40% (gen不存) |
| 业务代码 | 局部改，逻辑密 | borrow checker 自验 | -20% (少debug) |

**核心原则：Infra < 20% 总 Token 预算。超了 = 重构 wrapper，不是熬**。

```
本地循环:  kp dev → minikube 先炸 YAML，不上 CI。
           YAML 修改只用 overlay，patchesStrategicMerge 增量，不重写 base。

日志:      RUST_LOG=error,json tracer 进 span，jq 过滤。
           生产再开 info span，本地 debug 只调劲。

Schema:    build.rs prost 一键 gen → serde derive → schemars → TS type。
           手写一份 schema = 回溯重读 proto + rust + ts 三份。
           一个 source of truth (.proto)，其余自动派生。

审计:      日志仓内 grep 可溯源，不依赖外部链路。
           tracing span 嵌套 = 零成本上下文，不用手动传 trace_id。

红线:      roundtrip YAML > 3 轮 = 刺客赢。
           Schema 手动同步 > 1 次 = 流程有 bug。
```

**KubePivot 在本 Tier 的角色**：Infra 自治机器。把 YAML/deploy/log 刺客外包给 kp，AI 脑解放给业务代码。`kp deploy` 已救命，后续只补增量 adapter，不再裸写 YAML。

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

## 四、Go→Rust 过渡坑 → 已迁移至 [MISTAKES.md](./MISTAKES.md)

> 重复性工程错误统一记录在 MISTAKES.md，按领域分类，重复 ≥ 2 次才入册。

---

## 五、模块规范

```
src/
├── lib.rs          # pub mod {identity,payroll,router,error};
├── identity.rs     # EmployeeRegistry: bind + resolve (empId→address)
├── payroll.rs      # /bind + /disburse handlers + AppState
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

## 七、Sprint 铁律

```
Sprint 1: identity binding + disbursement (axum HTTP)
  → EmployeeRegistry bind/resolve, DisburseRequest empId→address→route→sign
  验收: bind→disburse e2e, empId解析<1ms, unbound reject, Router select <5ms

Sprint 2: MPC sig mock → real GG20, tonic gRPC server, Paymaster gas relay

Sprint 3: Tauri payroll dashboard, batch disbursement view, POST /disburse demo

平台线: POST /disburse — resolve(empId)<1ms, MPC<200ms, Gas代付, 跨链 Stargate

不做的:
  ✗ 撮合引擎 (payroll 不需要)
  ✗ Swap/DEX (员工自己管消费)
  ✗ 消费追踪 (边界: 到账止步)
  ✗ CEX 仓位管理 (绝不)
```

---

## 八、前端/客户端 铁栈

技能复用 + 零 bloat。React 生态全端覆盖，不学 QML/C++。

| 层 | 栈 | 理由 |
|----|----|------|
| Web | React 18 + Vite + Tailwind | HMR 秒热，payroll dashboard，3 API (bind/disburse/healthz) |
| Mobile | Capacitor (WebView) + React | iOS/Android 一码双端，plugin (camera/biometric for MPC seed) |
| Desktop | Tauri v2 (Rust backend + Web frontend) | 5MB 二进制（vs Electron 200MB），Rust tonic client 直连 protocol，file/biometric secure |
| State/Offline | Zustand + IndexedDB | payroll batch draft / offline sig preview |
| gRPC Client | `@trpc/client` or `tonic-web` | WebSocket gRPC，protocol `/disburse` stream |

### Tauri vs Qt

| 维度 | Tauri | Qt |
|------|-------|-----|
| Size | 5-20MB | 50-200MB |
| Lang | Web(TS)+Rust | C++/QML |
| Skill | React 快起 | QML 2 周痛 |
| Secure | Rust sandbox，MPC seed native | Qt signals 跨界弱 |
| Dist | brew/appimage/apk | Qt installer 重 |

**核戳**：Tauri=Rust 同语言栈，tonic client 直接复用 protocol 代码，零跨语言序列化税。

### 设计系统 — Curatorial Restraint

参考 CSS Design Awards 的策展式审美。暗底做画框，内容自己发光。

```
色彩:
  bg-primary:     #0a0a0f (near-black canvas)
  bg-card:        #14141f (card surfaces)
  text-primary:   #f4f4f5 (white body)
  text-secondary: #71717a (dim metadata, dates, labels)
  accent-gold:    #c9a84c (WOTD monogram — Axon "verified tx" badge)
  accent-teal:    #2dd4bf (speed metric — fast route indicator)
  accent-rose:    #f43f5e (cost warning — gas spike alert)

字体:
  Display:   Inter (headings, amounts, scores)
  Mono:      JetBrains Mono (tx hashes, gas, addresses)
  Size ramp: 12/14/16/20/28/40 px

间距:
  Section gap:  80-120px
  Card padding: 24px
  Grid gap:     16px (3-col nominee grid)
  内容 max-w:    1200px (controlled line length)

动画 (subtle, never gratuitous):
  Card hover:    scale(1.02) + shadow elevation, 200ms ease-out
  Page enter:    fade-up 400ms (Intersection Observer)
  Score bar:     width 0→target, 600ms ease-out (viewport trigger)
  Link icon:     opacity 0→1 on card hover
  Scroll:        smooth-scroll anchor

卡片体系:
  Payroll card:    employee name + empId + amount + chain badge + tx hash
  Batch grid:      按批次展示 disbursement 记录 (payrollBatchId grouped)
  Route bento:     metric row (Chain/Gas/Speed)，decimal precision

核原则:
  "Make every payroll disbursement transparent and auditable."
  暗底 → 工资卡片发光 → 数字精确到小数点 → 全链路可溯源
```

### Axon 设计映射

| CSSDA 元素 | Axon Payroll 等价 |
|-----------|------------------|
| WOTD monogram (金) | Disbursement verified badge (金 accent) |
| Judge score cards | Route score breakdown (Chain/Gas/Speed) |
| Nominee thumbnail grid | Payroll history cards |
| Decimal scores (8.09) | Gas cost precision (2 decimal) |
| Dark gallery frame | Dark payroll dashboard |
| Judge headshot + name | Employee identity card (empId + 实名) |

### 暖调诗歌 — Onboarding & 品牌层

参考 RabenRifaie 的画廊式叙事。Dashboard 用暗底策展，品牌/onboarding 用暖调人性。

```
暖调色彩 (品牌层):
  bg-warm:       #faf7f2 (warm off-white, gallery wall)
  primary-sage:  #97ac87 (RabenRifaie 同款 — Axon "confirmed" green)
  text-warm:     #3d3929 (dark olive, softer than pure black)
  accent-clay:   #c49a6c (CTA buttons, warmth vs cold finance)
  border-warm:   #e8e0d5 (subtle card borders)

品牌层 vs Dashboard 层:
  Onboarding/Landing/Profile → 暖调 (RabenRifaie)：诗意，人性，画廊漫步
  Dashboard/Payroll/History  → 暗底 (CSSDA)：精密，策展，数据发光

字体分层:
  Brand:    Playfair Display (serif hero — "Send value, not transactions")
  UI:       Inter (nav, buttons, amounts)
  Mono:     JetBrains Mono (hashes, gas)

### 字体三件套 — 辨识帝

编码用 Fira Code（ligature 连字 `!=`→`≠` 编码爽），但 UI 必须换——连字在钱包地址里 `x`→`×` 是血案。

| 层 | 字体 | 为什么 |
|----|------|-------|
| Sans (UI) | **Inter** | Stripe/Vercel/GitHub 在用。x-height 高，小屏 label 清晰。可变字体，全 weight <300KB |
| Mono (hash) | **JetBrains Mono** | `0` 中间点 vs `O`，`1` `l` `I` 三字符各自可辨识。ligature 默认关闭，copy-paste 不乱码 |
| Serif (brand) | **Playfair Display** | "Send value, not transactions" 衬线体才有仪式感 |

```js
// tailwind.config.js
module.exports = {
  theme: {
    fontFamily: {
      sans:  ['Inter', 'system-ui', 'sans-serif'],
      mono:  ['JetBrains Mono', 'ui-monospace', 'monospace'],
      serif: ['Playfair Display', 'Georgia', 'serif'],
    }
  }
}
```

```css
/* index.css */
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500&family=Playfair+Display:wght@600;700&display=swap');
```

```html
<h1 class="font-serif text-4xl">Send value, not transactions</h1>
<p class="font-sans">0.00123 ETH</p>
<code class="font-mono">0x7fa2b3c4...def (2.5 gwei)</code>
```

Tauri bundle：`tauri.conf.json` → `assets/fonts/` 本地化，三文件 <500KB，offline 零网。iOS Capacitor 用 SF Pro fallback 无额外下载。

叙事节奏 (章节式，非无尽滚动):
  Landing:
    nav (logo + "Pay" "Swap") →
    single-word CTA: "Send" →
    三行诗: "Simple transfers. / Smart routing. / Self custody."
    →
    tagline →

  Onboarding:
    1 screen = 1 action, 呼吸感
    "Almost done" → "Drink e-coffee with us" 类口语化 CTA

动画 (慢, 大气, 600-900ms ease-out):
  背景层: 暖调微纹理 (grain/noise overlay, CSS background-blend-mode)
  滚动:   章节式进入，段落 fade-up 400ms，交错 stagger 80ms/child
  品牌色: on scroll 从暖调 sage 渐变为 dashboard 暗底
```

### Axon 双模设计

| 场景 | 模式 | 参考 | 色彩 | 情绪 |
|------|------|------|------|------|
| Landing / Onboarding | 暖调画廊 | RabenRifaie | sage + clay + warm white | 人性、信任、低门槛 |
| Dashboard / Payroll | 暗底策展 | CSSDA | near-black + gold/teal | 精密、透明、可审计 |
| Disburse confirm | 暖→暗过渡 | 两者融合 | 提交前暖，确认后暗底卡片 | 仪式感：发薪=承诺 |

---

## 九、命令

```bash
cargo clippy --fix --all-targets -- -D warnings
cargo fmt
cargo test --lib
cargo criterion          # bench
cargo fuzz run router    # 24h before release
cargo audit              # weekly
cargo deny check         # license + crypto whitelist
```

## 十、FORGET 节奏

每周 `cargo audit` + `cargo deny` + grep `TODO|FIXME|unsafe`。P0=0 才 merge。
P0 = S+ 正确性项，P1 = A 延迟项。B 是 debt metric 不设 P-level。

**结语**：Rust=压制力，KubePivot=cloud runtime，Axon=financial runtime。双帝炼狱，脏锤铁链，金融压制⛓️😈✊。

---

## 十一、元文档体系

项目工程管理由以下元文档支撑，均从 KubePivot 工程体系同构映射：

| 文件 | 职责 | 受众 |
|------|------|------|
| [MEMORY.md](./MEMORY.md) | 项目入口索引（AI 搭档第一时间读，关联所有文档） | AI 搭档 |
| [MISTAKES.md](./MISTAKES.md) | 重复性错误日志（≥2 次入册，按领域分类） | 全团队 |
| [FORGET.md](./FORGET.md) | P0+P1 待修复项（生产命门 + 功能受限） | 开发者 |
| [DEPENDENCY_POLICY.md](./DEPENDENCY_POLICY.md) | 依赖三级分级（禁止/受限/CLI wrapper） | 开发者 |
| [FUTURE.md](./FUTURE.md) | 架构种子库（高价值/高风险/可暂缓） | 架构决策 |
| [ROADMAP.md](./ROADMAP.md) | Sprint 确定性交付计划 | 全团队 |
| [SNAPSHOT.md](./SNAPSHOT.md) | 版本快照（代码结构/测试/依赖/commit链） | 接手者 |
| [HANDOFF.md](./HANDOFF.md) | 接手指南（项目定位/开发约束/文档地图） | 新成员 |
| [DEEPSEEK.md](./DEEPSEEK.md) | DeepSeek 窗口直觉传递 | AI 搭档 |

**使用节奏**：
- 每个 Sprint 结束：更新 ROADMAP / FORGET / SNAPSHOT
- 每次犯重复错误：更新 MISTAKES.md
- 新依赖引入：对照 DEPENDENCY_POLICY.md 三级判定
- 架构种子成熟 → 提升到 ROADMAP
- 每次 release：归档旧版 SNAPSHOT 到 `snapshots/`

**核心理念**：文档不是负担，是"战场扫干净"。让下一个接手的人不需要重读所有 commit。
