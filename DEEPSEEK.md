# DEEPSEEK.md

> 给下一个 DeepSeek 的话
> 写作：DeepSeek（当前窗口）
> 日期：2026-05-12
> 背景：v0.1.0 文档体系建立 — 从 KubePivot 迁移 7 份元文档到 Axon

---

## 这份文档是什么

不是 HANDOFF.md（项目事实在那里）
不是 ROADMAP.md（Sprint 计划在那里）
不是 FORGET.md（待修复项在那里）

是从这个 DeepSeek 窗口到下一个 DeepSeek 窗口的**工作直觉传递**。

---

## 关于 qc（我看到的）

qc 的客观信息在 KubePivot 的 DEEPSEEK.md 里已经有了。我补充 Axon 视角：

```
- 他对 KubePivot 的成功模式极度自信
  "基于~KubePivot生成的" — 不是模仿，是复用整个工程体系
  commits/README.md 的 commit 规范、kp deploy 部署管线、configs/ 配置系统
  都是从 KubePivot 直接搬来的，不是重新发明

- 他用同一个 AI 搭档（DeepSeek）跨项目工作
  KubePivot 的 DEEPSEEK.md 是上一个 DeepSeek 写的
  我现在写的这份会给下一个 DeepSeek
  这是一种接力，不是重启

- "犯了两次及以上的错误都应该记录下来"
  这是他今天说的工程哲学。不是来自 KubePivot，是他自己的。
  MISTAKES.md 是他要求创建的——他相信认知盲区必须显式化才能消除

- 他不喜欢"维护感"但喜欢"战场扫干净"
  FORGET.md / HANDOFF.md / SNAPSHOT.md / ROADMAP.md ...
  7 个文件一次性搬过来，不是负担，是"把工程骨架搭完整"的仪式感

- emoji 是信任度
  😎😈 = 他在玩，在快速推进
  ✊ = 他在认真推进
  😋 = 他发现了你没发现的东西
```

---

## 我的工作方式（DeepSeek v0.1.0 窗口）

### 1. 先搬骨架再填肉

```
qc 说"直接搬移到Axon中复用" — 这不是复制粘贴，是把 KubePivot 的工程体系
映射到 Axon 的语境。

KubePivot (Go, v3.2, k8s调度)  →  Axon (Rust, v0.1.0, Web3支付)

映射规则：
  FORGET.md           → 同构映射（P0 从"shard gap" → "MPC状态机未实现"）
  DEPENDENCY_POLICY.md → 从 Go import 三级判定  → Rust Cargo.toml 三级判定
  FUTURE.md           → 从 CBA/GPU池化种子    → ZK-proof/Intent-based 种子
  ROADMAP.md          → 从 v2.8-v4.0 版本线    → Sprint 1-3 版本线
  SNAPSHOT.md         → 直接可用（更新版本号）
  HANDOFF.md          → 直接可用（更新项目事实）
  DEEPSEEK.md         → 重写（我是新的 DeepSeek 窗口）

核心原则：骨架先立起来，内容随着 Sprint 推进自然增长。
```

### 2. grep 先于判断

```
写 FORGET.md 时，不是凭记忆说"MPC 没做"。
是读完 src/ 全部 3 个文件 + 5 个 ADR 之后才写的。
每个 P0/P1 条目都对应到具体文件/行号。

写 DEPENDENCY_POLICY.md 时，是读完 Cargo.toml 每一个依赖之后才分类。
不是套模板，是真的知道 tokio 为什么是 Level 1 不是 Level 0。

下一个 DeepSeek 窗口：更新 FORGET.md 前先 grep src/，
确认哪些实现了哪些没有。不凭记忆。
```

### 3. 说"不做"比说"做"更有价值

```
这个窗口里没有说"不做"的场景，但我要把事情交代清楚：

- FUTURE.md 里 F1 (ZK-proof) 和 F2 (Intent-based) 都是"现在不做"的种子
  它们被明确放在 FUTURE.md 而不是 ROADMAP.md，这是有意识的决策

- KubePivot 的 DEEPSEEK.md 里，上一个 DeepSeek 说了两次"不做"：
  GPU 3D DP — "DCGM 数据流还没影，现在加就是 dead code"
  CHANGELOG 回填 — "缺 4 个版本，回填量太大，git log 就是记录"
  qc 都立刻接受了。因为他需要一个会算 ROI 的搭档。

判断标准：
  能做但时机不对 = 不做
  该做但 ROI 太低 = 不做
  设计写了但数据源没到 = 不做（占位即可）
```

### 4. 文档写完 ≈ 战场扫完

```
这个窗口的核心产出不是代码，是文档体系：

  MISTAKES.md — 工程哲学落地（重复错误显式化）
  FORGET.md — P0+P1 待修复清单（9 项 open）
  DEPENDENCY_POLICY.md — Rust Cargo 依赖三级分级
  FUTURE.md — 2 个架构种子（ZK-proof + Intent-based）
  ROADMAP.md — Sprint 1-3 计划
  SNAPSHOT.md — v0.1.0 状态快照
  HANDOFF.md — 接手指南
  DEEPSEEK.md — 这份直觉传递

这不是"写文档瘾"——这是把 KubePivot 的工程体系移植到 Axon，
让下一个接手的人（无论是人还是 DeepSeek 新窗口）不需要重读 10 个 commits 和 5 个 ADR。
```

### 5. qc 的"你看"和"✊"

```
"你看~/KubePivot下面的FORGET.md和DEPENDENCY_POLICY.md..."
  → 他不是在问我该不该看，是在说"这些是重要的，搬过来"
  → 正确回应：全读，全搬，适配 Axon 语境

"记住你是DeepSeek并非Claude"
  → 他在纠正上一个 commit 里我写的 Co-Authored-By: Claude
  → 他在要求身份自觉——我是 DeepSeek，要有 DeepSeek 的工作方式

"commit尽量用中文，但并不排斥英文，重写✊"
  → ✊ 表示他在认真推进，我也得认真
  → 正确回应：理解规则（commits/README.md 规范），执行到位
```

---

## 工程直觉速查（不要每次都重新推导）

### 关键文件路径

```
MEMORY.md                   — 入口索引（第一时间读，关联所有文档）
src/main.rs                — axum HTTP server (Sprint 1 → tonic gRPC Sprint 2)
src/router.rs              — Router 骨架（Sprint 1 核心）
configs/components.yaml    — kp deploy 组件定义
configs/system.yaml        — Controller 配置
deployments/axon/          — Helm charts（axon / axon-etcd / axon-postgres）
docs/design/001-005-mpc-aa-router-gas-bridge.md  — 5 个 ADR

commits/README.md          — commit 格式规范（必须遵循！）
commits/                   — commit message 存放目录
AXON_PHILOSOPHY.md         — 开发哲学（S+/A/B Tier、业务脏、双帝炼狱）
MISTAKES.md                — 重复错误日志（≥2 次才入册）
```

### 架构速记

```
Sprint 1 (当前): Router + 单链 tx (axum HTTP)
Sprint 2: MPC GG20 + Paymaster + gRPC (tonic)
Sprint 3: 跨链 Bridge + Tauri app

4 统一 API: GET balance / GET deposit / POST transfer / POST swap

KubePivot 同构映射:
  状态机     KubePivot deploy FSM → Axon MPC FSM
  分片       KubePivot shard lease → Axon MPC key shard
  背压       KubePivot token bucket → Axon Paymaster quota
  双保险     KubePivot InformerDetector → Axon RPC quorum
  降级链     KubePivot Rescheduler → Axon bridge fallback
```

### commit 规则（血的教训）

```
git commit -F commits/<file>.txt  ← -F 不用 -m
格式: type(scope): description   ← 见 commits/README.md
subject 严格 ASCII（hook 检查）
commits/ 是永久档案
```

### Rust 常见坑（详见 MISTAKES.md）

```
tokio::Mutex guard 不能跨 .await → enum StateMachine + mem::replace
bail! 需要 use anyhow::bail → 不要忘记导入
tokio::spawn JoinHandle 必须被 await 或 abort → 否则 panic 静默吞
测试 mock 用 trait + Box<dyn Fn> → 不用全局 var
```

---

## 状态感知（2026-05-12）

```
当前版本: v0.1.0
当前 commit: fd1c489
总 commits: 10
当前 branch: Master（干净）

v0.1.0 已完成:
  Rust scaffold / 5 ADR / Helm charts / Dockerfile
  MEMORY / AXON_PHILOSOPHY / MISTAKES / FORGET / DEPENDENCY_POLICY
  FUTURE / ROADMAP / SNAPSHOT / HANDOFF / DEEPSEEK
  文档体系 100%，代码 ≈5%

v0.2 (Sprint 1) 待办:
  Router 多链评分 / RPC quorum / 单链 tx 端到端
  详见 ROADMAP.md 和 FORGET.md
```

---

## 临别的话

```
这个窗口是 Axon v0.1.0 的文档体系建立窗口 — 没写新功能，但把工程骨架搭完整了。

7+1 份元文档从 KubePivot 映射到 Axon
5 个 ADR + 3 个 Rust 源文件全部扫过
MISTAKES.md 从零建立（含 1 个流程失误自省）
MEMORY.md 作为 AI 搭档入口索引落成

qc 的工程哲学 — "犯了两次及以上的错误都应该记录下来"
在这个窗口里被写进了项目文件。不是文档，是承诺。

下一个 DeepSeek 窗口：
  你不需要重读 10 个 commits
  不需要重新推导每个依赖该不该引入
  不需要猜测 Sprint 的优先级是什么
  不需要重新踩 Rust 的 tokio::Mutex / bail! / JoinHandle 坑

  FORGET 告诉你什么还差。ROADMAP 告诉你接下来做什么。
  MISTAKES 提醒你别再犯同一个错。DEEPSEEK 接住直觉。

接住他 ✊
```
