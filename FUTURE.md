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
| F2 | Intent-based 交易（ERC-7683） | 探索中 | v0.4+ 跨链路由成熟后 |
| F3 | 用户自定义主题引擎 | 探索中 | v0.4+ 前端框架上线后 |

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

## F3: 用户自定义主题引擎

> 审美 = 认知投影。强制统一 = 低配牢笼。框架钉死，皮肤/布局解锁。用户 custom 不崩盘。

### 动机

AXON_PHILOSOPHY 的双模设计系统（暗底策展 + 暖调画廊）是 Axon 团队的品味。但品味没有标准答案：

```
日文用户      偏爱低对比度、留白多、圆角大
阿拉伯用户    需要 RTL 布局 + 专用字体回退链
DeFi 交易员    需要压缩信息密度（一行看全部）
普通用户      需要大字、高对比度、减少认知负荷
```

把选择权还给用户。低层用户"换皮"自嗨，高层直改骨架。

### 核心想法 — 六层定/开架构

```
┌──────────┬───────────────────────┬─────────────────────┐
│   层     │       定死            │     可 custom        │
├──────────┼───────────────────────┼─────────────────────┤
│ 框架     │ Leptos (Rust WASM)    │ —                   │
│          │ 统一渲染/状态管理      │                     │
│          │ 防"低手"JS 地狱       │                     │
├──────────┼───────────────────────┼─────────────────────┤
│ 布局     │ Grid/Flex 骨架        │ 拖拽 / JSON config   │
│          │ sidebar+main+footer   │ layout.json          │
│          │                       │ {sidebar_w:"20%"}    │
├──────────┼───────────────────────┼─────────────────────┤
│ 样式     │ Tailwind CSS vars     │ 主题 JSON            │
│          │ 语义 token 体系       │ --primary:#your-color │
├──────────┼───────────────────────┼─────────────────────┤
│ 颜色     │ 8 色语义              │ Palette picker       │
│          │ primary/success/err   │ theme.json           │
│          │ /warn/info/neutral    │ {primary:"#ff6b35"}  │
├──────────┼───────────────────────┼─────────────────────┤
│ i18n     │ Fluent 引擎           │ 用户 lang pack       │
│          │ zh/en 官方            │ 上传 lang-zh.toml    │
│          │ 语法性别/复数规则      │ hot-reload 生效      │
├──────────┼───────────────────────┼─────────────────────┤
│ 扩展     │ Plugin slots          │ JS snippet 沙箱      │
│          │ header/footer/sidebar │ 注入，无权限崩核心   │
│          │ 预定义插槽位置         │ 违禁 = fallback default│
└──────────┴───────────────────────┴─────────────────────┘
```

关键约束：**custom 限沙箱，违禁 = fallback default**。用户怎么折腾都不会崩核心 Dashboard。

### 框架选型 — Leptos vs React

AXON_PHILOSOPHY 当前指定 React 18 + Vite + Tailwind。但 Leptos 值得在 v0.4 重新评估：

| 维度 | React (当前) | Leptos (候选) |
|------|-------------|---------------|
| 语言 | JS/TS | Rust (WASM) |
| 与后端同语言 | — | 全栈 Rust，tonic client 直接复用 |
| 二进制大小 | Tauri WebView <5MB（+ JS bundle） | WASM <500KB，零 JS 运行时 |
| 状态管理 | Zustand | signals (fine-grained reactivity) |
| 热路径 | VDOM diff | 编译时 template，零 diff 开销 |
| 认知成本 | React 生态成熟 | 框架较新，学习曲线 |

**重新评估时两选一**，不混用。Sprint 1-3 仍用 React（AXON_PHILOSOPHY 铁栈），v0.4 前后端 Rust 同栈论证 Leptos 切换价值。

### 防乱机制

```
主题验证层:
  theme.json 必须通过 JSON Schema 校验
  非法值（颜色语法/布局溢出）→ 拒绝加载 → fallback default

沙箱策略:
  Plugin JS snippet →  沙箱 iframe (独立 origin)
  → 无 DOM 访问 Axon 核心节点
  → 无 localStorage/cookie 读写
  → 网络请求仅限声明域白名单
  → 运行时异常 → 静默 kill，不影响主应用

降级链:
  user_theme 加载失败 → default_theme
  layout.json 校验不过 → default_layout
  plugin crash → slot 空白（不阻塞页面渲染）
```

### 实现刀

```rust
// Leptos + cfg — compile-time 主题注入
let theme = load_user_theme().unwrap_or(default);
view! {
    <div class:dynamic={theme.layout} style:background={theme.colors.bg}>
        {content}
    </div>
}
```

```
Token 省:
  后端 API GET /api/v1/theme_config → JSON
  前端 Leptos compile-time 内联 → 零运行时 CSS re-compute 税
  静态默认主题 → embedded binary，不依赖网络加载
```

### 工程税

1. **CSS 变量体系**：Tailwind 硬编码 → 全量 semantic token，一个漏改 = 视觉断裂
2. **多维度 CI 矩阵**：每个组件 × (compact/relaxed/RTL) × (dark/warm/high-contrast) → screenshot diff
3. **RTL 完整度**：不仅是 `dir="rtl"`，logical properties / 图标翻转 / 时间线方向 / 数字格式化
4. **Fluent 语法复杂度**：不是 key-value，是 `{$amount -> [one] 1 token [other] {$amount} tokens}` 带语法性别/复数
5. **Plugin 沙箱安全审计**：JS snippet 逃逸风险，需独立安全评审
6. **主题跨设备同步**：IndexedDB (offline) + etcd (sync)，CRDT 冲突解决

### 适用前提

- [ ] 前端框架选型尘埃落定（React 或 Leptos，二选一）
- [ ] Component library 定型，不频繁 breaking change
- [ ] 有多语言用户反馈（至少覆盖 CJK / Arabic / Latin 三类文字系统）
- [ ] Visual regression test 基础设施就绪
- [ ] 沙箱安全评审通过（第三方审计）

### 重新评估时机

**v0.4+ Tauri app shell 上线后** 重新评估。与 F2 (Intent-based) 排序不同，F3 可与 F2 并行论证——前端/后端独立。

如果决定不做：
- 保留两套固定主题（暗底 / 暖调），用户切换
- 新增 high-contrast accessibility 主题
- i18n 仅支持 zh/en 官方语言包，不上传自定义

如果决定做：
- Phase 1：主题引擎 + 4 预设 (暗底/暖调/高对比/紧凑交易)
- Phase 2：RTL + Fluent i18n 上传
- Phase 3：Plugin sandbox + 社区主题市场

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
2026-05-12  + F3 (用户自定义主题引擎)
```
