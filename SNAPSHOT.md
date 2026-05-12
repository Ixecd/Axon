# SNAPSHOT — Axon v0.1.0

> 编写日期：2026-05-12
> Last commit: fd1c489
> Total commits: 10
> Co-Authored-By: DeepSeek

---

## 一、版本与 commit

```
当前 branch:        Master
当前 commit:        fd1c489
upstream:           (待定)
total commits:      10
latest tag:         无
```

**v0.1.0 commit 链**：

| Commit | 内容 |
|--------|------|
| `fd1c489` | docs(mistakes): add centralized repeat-error tracking log |
| `4a2dc07` | feat(docs): AXON_PHILOSOPHY v3 — design system + frontend stack + positioning |
| `8fa03dd` | feat(Axon): init Rust scaffold — ADR 001-005 + RUST_PHILOSOPHY v2 |
| `36c8ea1` | feat: Initialize Axon scaffolding |
| `ffada5e` | chore: init project by kp |

---

## 二、代码结构

```
Axon/
├── src/
│   ├── main.rs          # axum HTTP server — /healthz + /
│   ├── lib.rs           # pub mod router (mpc/paymaster/bridge TODO Sprint 2)
│   └── router.rs        # Router struct 骨架 (Sprint 1 原型)
│
├── docs/design/
│   ├── axon-design.md   # Go 时代技术设计（历史参考）
│   ├── 001-mpc.md       # ADR 001: MPC GG20 t-of-n
│   ├── 002-aa.md        # ADR 002: ERC-4337 AA
│   ├── 003-router.md    # ADR 003: 多链 Router
│   ├── 004-gas.md       # ADR 004: Gas Vault
│   └── 005-bridge.md    # ADR 005: Cross-chain Bridge
│
├── deployments/axon/    # Helm charts（axon / axon-etcd / axon-postgres）
├── configs/             # KubePivot 部署配置
├── build/docker/axon/   # Dockerfile（multi-stage, scratch）
├── commits/             # 工程档案
├── scripts/             # make-rules + 运维脚本
│
├── Cargo.toml           # Rust 项目配置（axum + tonic + tokio）
├── Makefile             # dev / test / lint / image / deploy
├── MEMORY.md            # 项目入口索引（AI 搭档第一读）
├── AXON_PHILOSOPHY.md   # 开发哲学 & 规范指南
├── MISTAKES.md          # 重复性错误日志（按领域分类）
├── FORGET.md            # P0+P1 待修复项
├── DEPENDENCY_POLICY.md # 依赖管理三级分级
├── FUTURE.md            # 架构种子库
├── ROADMAP.md           # Sprint 交付计划
├── HANDOFF.md           # 接手指南
├── DEEPSEEK.md          # DeepSeek 窗口直觉传递
└── README.md
```

---

## 三、测试状态

```
当前测试: cargo test --lib 仅覆盖 src/ 骨架代码
完整测试套件: 尚未建立（Sprint 1 开始填充）
bench: criterion 已配置，无实际 bench case
fuzz: proptest 已配置，无实际 fuzz target
```

---

## 四、依赖清单（Cargo.toml）

| 依赖 | 级别 | 用途 |
|------|------|------|
| tokio | Level 1 | 异步运行时 |
| axum | Level 1 | HTTP server |
| tonic / prost | Level 1 | gRPC server + protobuf |
| serde / serde_json | Level 1 | 序列化 |
| tracing / tracing-subscriber | Level 1 | 可观测性 |
| thiserror / anyhow | Level 1 | 错误处理 |
| bytes | Level 1 | 零拷贝 buffer |
| reqwest | Level 1 | Oracle HTTP client |
| rand | Level 1 | 确定性 seeded RNG |

当前 0 个 Level 0（禁止）依赖，0 个 Level 2 CLI wrapper 依赖。

---

## 五、关联文档

- [FORGET.md](FORGET.md) — P0+P1 待修复清单（9 项 open）
- [HANDOFF.md](HANDOFF.md) — 接手指南
- [ROADMAP.md](ROADMAP.md) — Sprint 交付计划
- [FUTURE.md](FUTURE.md) — 架构种子库
- [DEPENDENCY_POLICY.md](DEPENDENCY_POLICY.md) — 依赖管理政策
- [MISTAKES.md](MISTAKES.md) — 重复性错误日志
- [AXON_PHILOSOPHY.md](AXON_PHILOSOPHY.md) — 开发哲学 & 规范指南
- [docs/design/](docs/design/) — ADR 001-005

---

## 编辑记录

```
2026-05-12  v0.1.0 snapshot — 10 commits, fd1c489 HEAD, 文档地图建立
```
