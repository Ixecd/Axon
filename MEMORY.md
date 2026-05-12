# MEMORY.md — Axon 项目入口索引

> 这是 AI 搭档打开项目后**第一时间读的文件**。
> 读完此文件 → 按文档地图索引到具体文档 → 建立完整项目心智模型。
> 最后更新：2026-05-12 | v0.1.0

---

## 一、项目身份（30 秒速览）

```
Axon = Web3 通用极简交互枢纽
核心理念：平台不持不撮，self-custody
一句话：transfer(from, to, amount, chain) — 一切交易都是 transfer 的壳
技术栈：  Rust 1.80+ / axum→tonic / MPC+AA / KubePivot 部署
当前版本：v0.1.0 — 文档完整，代码 ≈5%（Router 骨架 + HTTP server）
```

---

## 二、文档地图（按阅读顺序）

### 第一圈：核心理解（必读，按此顺序）

| 序号 | 文件 | 回答的问题 | 读完时长 |
|------|------|-----------|----------|
| 1 | `MEMORY.md` | 我在哪？这个项目是什么？文档怎么找？ | 你正在读 |
| 2 | [README.md](./README.md) | 项目对外介绍，定位和价值主张 | 3 min |
| 3 | [AXON_PHILOSOPHY.md](./AXON_PHILOSOPHY.md) | 怎么写代码？Tier 铁锁 / 业务脏 / 双帝炼狱 | 15 min |
| 4 | [HANDOFF.md](./HANDOFF.md) | 现在能做什么？开发约束？工作流？ | 5 min |
| 5 | [SNAPSHOT.md](./SNAPSHOT.md) | 精确到 commit 的代码结构和测试状态 | 5 min |

### 第二圈：工程管理（知道项目怎么管）

| 文件 | 回答的问题 |
|------|-----------|
| [ROADMAP.md](./ROADMAP.md) | Sprint 1-3 分别交付什么？平台线/壳线怎么分？ |
| [FORGET.md](./FORGET.md) | 什么东西还没做？（P0 命门 / P1 受限） |
| [FUTURE.md](./FUTURE.md) | 什么种子现在不做、以后可能做？ |
| [DEPENDENCY_POLICY.md](./DEPENDENCY_POLICY.md) | 新 crate 能引入吗？三级判定怎么走？ |
| [MISTAKES.md](./MISTAKES.md) | 哪些坑踩过 ≥2 次？认知盲区在哪里？ |

### 第三圈：设计决策（理解为什么这么做）

| 文件 | 回答的问题 |
|------|-----------|
| [docs/design/001-mpc.md](./docs/design/001-mpc.md) | MPC GG20 t-of-n 签名协议——为什么选这个？ |
| [docs/design/002-aa.md](./docs/design/002-aa.md) | ERC-4337 账户抽象——Session key / Paymaster 设计 |
| [docs/design/003-router.md](./docs/design/003-router.md) | 多链路由评分公式——Speed/Cost/Safety 权重 |
| [docs/design/004-gas.md](./docs/design/004-gas.md) | Gas Vault——多签金库 / 安全系数 / 用户限额 |
| [docs/design/005-bridge.md](./docs/design/005-bridge.md) | 跨链桥安全——Stargate 主 / CBridge fallback |
| [docs/design/axon-design.md](./docs/design/axon-design.md) | Go 时代完整技术设计（历史参考，非当前架构） |

### 第四圈：AI 搭档专属

| 文件 | 回答的问题 |
|------|-----------|
| [DEEPSEEK.md](./DEEPSEEK.md) | 上一个 DeepSeek 窗口留下了什么直觉？qc 是什么风格？ |
| [commits/README.md](./commits/README.md) | commit 格式规范——不遵守会被 hook 拒绝 |

---

## 三、架构速览（一张图理解项目）

```
用户 (Axon App / Wallet)
  │
  ├── GET  /api/v1/balance
  ├── GET  /api/v1/deposit
  ├── POST /api/v1/transfer
  └── POST /api/v1/swap
        │
        ▼
┌──────────────────────────────────────┐
│         Axon Protocol (Rust)         │
│                                      │
│  ┌──────────┐  ┌──────────┐         │
│  │  Router  │  │   MPC    │         │
│  │ 多链决策  │  │ GG20签名  │         │
│  │ 3中2 quorum│  │ t-of-n   │         │
│  └──────────┘  └──────────┘         │
│                                      │
│  ┌──────────┐  ┌──────────┐         │
│  │Paymaster │  │  Bridge  │         │
│  │ Gas代付   │  │ 跨链桥接  │         │
│  │ jitter缓冲│  │ finality  │         │
│  └──────────┘  └──────────┘         │
│                                      │
│  ┌──────────────────────────┐       │
│  │        etcd (state)      │       │
│  │  idempotency / nonce TTL │       │
│  └──────────────────────────┘       │
└──────────────────────────────────────┘
        │
        ▼
┌──────────────────────────────────────┐
│        web3-blitz (底层引擎)         │
│  链交互 / HD钱包 / 签名 / 桥接       │
└──────────────────────────────────────┘
```

**三层架构**：
1. **Top**：Axon App（React/Tauri/Capacitor）— 用户界面
2. **Middle**：Axon Protocol（本 crate）— 路由 / 签名 / Gas / 桥接
3. **Bottom**：web3-blitz — 链上执行引擎

**KubePivot 同构映射**（贯穿所有设计）：
```
状态机     KubePivot deploy FSM  → Axon MPC FSM (Idle→Signing→Done)
分片       KubePivot shard lease → Axon MPC key shard
背压       KubePivot token bucket → Axon Paymaster quota
双保险     KubePivot InformerDetector → Axon RPC quorum 3中2
降级链     KubePivot Rescheduler → Axon bridge fallback + gas jitter
```

---

## 四、代码结构速查

```
src/
├── main.rs          # axum HTTP server — /healthz + /
├── lib.rs           # pub mod router (mpc/paymaster/bridge TODO Sprint 2)
└── router.rs        # Router struct 骨架

当前 Sprint 1:
  src/router.rs ← 正在施工（多链评分 + RPC quorum）

Sprint 2:
  src/mpc.rs / src/paymaster.rs / src/bridge.rs ← 尚未创建
```

---

## 五、关键约定（踩坑前必看）

### commit 规范
```
git commit -F commits/<file>.txt   ← 不是 -m
格式: type(scope): description    ← 见 commits/README.md
subject 严格 ASCII
```

### Rust 常见坑
```
tokio::Mutex guard 不能跨 .await  → enum StateMachine + mem::replace
bail! 需要 use anyhow::bail;      → 别忘导入
tokio::spawn Handle 必须 await    → 否则 panic 静默吞
测试 mock 用 trait 不用全局 var    → 见 MISTAKES.md R01-R04
```

### 依赖引入
```
新增 crate → 查 DEPENDENCY_POLICY.md 三级判定
L0 禁止 / L1 受限需论证 / L2 CLI wrapper 直接通过
```

### 错误记录
```
犯两次 → MISTAKES.md 入册（根因+解法+计数）
MISTAKES.md 本身就是这条规则的产品
```

---

## 六、当前状态（2026-05-12）

```
版本:    v0.1.0
提交:    10 commits (fd1c489 HEAD)
分支:    Master（干净）
Sprint:  Sprint 1 — Router 多链评分 + RPC quorum
部署:    kp deploy → K8s (Helm charts 已就绪)

代码完成度:
  文档体系   ████████████████████ 100%
  Router    ██░░░░░░░░░░░░░░░░░░   5%
  MPC       ░░░░░░░░░░░░░░░░░░░░   0%
  Paymaster ░░░░░░░░░░░░░░░░░░░░   0%
  Bridge    ░░░░░░░░░░░░░░░░░░░░   0%

P0 待修复:  4 项 (MPC状态机 / dropout recovery / Paymaster / Bridge)
P1 待修复:  5 项 (Router评分 / RPC quorum / AA / chaos-mesh / etcd schema)
详见 FORGET.md
```

---

## 七、快速命令

```bash
cargo run              # 启动 HTTP server (localhost:8080)
make dev               # build + test + clippy + fmt
make build.release     # release build (LTO + panic=abort)
make image             # Docker 镜像构建
kp deploy              # 一键部署到 K8s
```

---

## 八、外部依赖

| 系统 | 关系 | 文档 |
|------|------|------|
| KubePivot (kp) | 部署工具链 + 工程体系同构源 | `~/KubePivot/` |
| web3-blitz | 底层链引擎（HD钱包/签名/桥接） | 外部 crate |
| Ethereum/Polygon | 目标链 | — |
| Stargate / CBridge | 跨链桥（API 接入） | — |

---

## 九、读完后你应该能回答

- [ ] Axon 是什么？不是什么？（不持不撮，非 CEX/DeFi/Bridge）
- [ ] 核心函数是什么？（`transfer(from, to, amount, chain)`）
- [ ] 当前 Sprint 在做什么？（Router 多链评分）
- [ ] 下一个文件该读哪个？（按文档地图顺序）
- [ ] commit 怎么写？（`git commit -F commits/<file>.txt`）
- [ ] 新 crate 怎么判？（查 DEPENDENCY_POLICY.md）
- [ ] 犯错怎么记录？（≥2 次 → MISTAKES.md）
