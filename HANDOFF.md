# HANDOFF — Axon v0.1.0

> 编写日期：2026-05-12
> Last release: v0.1.0
> Total commits: 10
> Co-Authored-By: DeepSeek

---

## 一、项目定位

**Axon** 是 Web3 通用极简交互枢纽 — "区块链中枢神经系统"。非托管 MPC+AA 支付协议，Rust 实现。

**核心论断**：整个项目归结为一个函数：`transfer(from, to, amount, chain)`。买币、swap、跨链、提现——全是 transfer 的不同壳。

**技术选型**：
- **Rust 1.80+** — 零成本抽象，无 GC，金融级正确性
- **axum → tonic** — Sprint 1 HTTP，Sprint 2 切 gRPC
- **不引入完整 MPC 协议实现** — 自研 GG20 状态机（见 DEPENDENCY_POLICY.md）
- **trait 注入 mock** — 不用全局变量（见 MISTAKES.md R04）

---

## 二、现在能做什么

### 当前可运行

```bash
cargo run     # axum HTTP server 启动在 :8080
curl /healthz # → "ok"
curl /        # → "Axon Protocol v0.1.0"
```

### 当前可部署

```bash
kp deploy     # 一键部署到 K8s（Helm charts + Docker image）
```

### 当前文档完整度

- 5 个 ADR（MPC / AA / Router / Gas / Bridge）— 100% 设计完成
- AXON_PHILOSOPHY.md — 开发规范 100% 完成
- MISTAKES.md / FORGET.md / DEPENDENCY_POLICY.md / FUTURE.md / ROADMAP.md — 全量到位
- **代码实现：≈5%**（Router 骨架 + HTTP server，MPC/Paymaster/Bridge 为零）

---

## 三、开发约束

### 3.1 commits/ 目录

commit 前写草稿到 `commits/`，`git commit -F commits/<file>.txt`。
格式遵循 `commits/README.md` 规范：`type(scope): description`（ASCII subject，不以 `.` 结尾）。

### 3.2 设计先行

新功能：`docs/design/<feature>.md` → 拍板 → 实施。ADR 在 `docs/design/` 下编号管理。

### 3.3 trait 注入 mock

```rust
// 不用全局 var 注入（Go 惯用模式在 Rust 里不工作）
trait OracleProvider {
    async fn gas_price(&self) -> Result<u64>;
}
// 生产: RealOracleProvider
// 测试: MockOracleProvider（注入谎报数据）
```

### 3.4 依赖审查

新增 crate → 对照 DEPENDENCY_POLICY.md 三级判定。详见该文件。

### 3.5 错误记录

重复 ≥ 2 次的错误 → [MISTAKES.md](MISTAKES.md)。
首次犯 → 观察。再犯 → 入册（根因+解法+重复计数）。

---

## 四、当前工作流

```bash
make dev          # cargo build + cargo test --lib + cargo clippy + cargo fmt --check
cargo run         # 启动 HTTP server (localhost:8080)
kp deploy         # 一键部署到 K8s
```

---

## 五、当前可工作状态

```
HEAD:            fd1c489
last commit:     docs(mistakes): add centralized repeat-error tracking log
total commits:   10
crate:           axon v0.1.0

直接依赖:
  tokio 1 (full)          # 异步运行时
  axum 0.8                 # HTTP server
  tonic 0.11 + prost 0.12  # gRPC server
  serde 1 + serde_json 1   # 序列化
  tracing 0.1              # 可观测性
  thiserror 1 + anyhow 1   # 错误处理
  bytes 1                  # 零拷贝 buffer
  reqwest 0.12             # Oracle HTTP client
  rand 0.8                 # 确定性 seeded RNG（非 thread_rng）

源码:
  src/main.rs / lib.rs / router.rs
  共 ~60 行 Rust 代码
```

---

## 六、文档地图

```
根目录:
  MEMORY.md (AI 搭档入口索引 — 第一时间读这个)
  HANDOFF.md / ROADMAP.md / SNAPSHOT.md / FORGET.md
  FUTURE.md / DEPENDENCY_POLICY.md / MISTAKES.md
  AXON_PHILOSOPHY.md / README.md

docs/design/ (设计文档 — ADR):
  001-mpc.md        — MPC GG20 t-of-n 签名协议
  002-aa.md         — ERC-4337 账户抽象
  003-router.md     — 多链路由评分
  004-gas.md        — Gas Vault 与波动风险
  005-bridge.md     — 跨链桥安全
  axon-design.md    — Go 时代技术设计（历史参考）

configs/
  components.yaml   — kp deploy 组件定义
  system.yaml       — Controller 配置
  resources.yaml    — 自愈资源声明
  project.env       — 部署环境变量

deployments/axon/   # Helm charts（axon / axon-etcd / axon-postgres）
commits/            # 工程档案（commit message 模板）
scripts/            # make-rules + 运维脚本
```

---

## 七、联系

```
作者:    qc (庆春镜像)
许可:    MIT (2026, Ixecd)
基于:    KubePivot (kp) 脚手架生成
```
