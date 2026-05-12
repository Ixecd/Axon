# Axon 工程错误日志

> 哲学：**犯了两次及以上的错误都应该记录下来。单次错误是偶然，重复错误是认知盲区——必须显式化才能消除。**
>
> 规则：重复次数 ≥ 2 才入册。首次犯→观察；再次犯→记录根因+解法，全团队显式共享。

---

## 目录
- [Rust 语言陷阱](#rust-语言陷阱)
- [infra→业务 转型陷阱](#infra业务-转型陷阱)
- [kp / KubePivot 工具链](#kp--kubepivot-工具链)
- [Kubernetes / Helm](#kubernetes--helm)
- [Docker / 镜像构建](#docker--镜像构建)
- [工程流程](#工程流程)

---

## Rust 语言陷阱

### R01: tokio::Mutex guard 跨 `.await` 持锁死锁

- **首次日期**: 2026-05-11
- **症状**: `let guard = self.sm.lock().await;` 后在 `.await` 点处编译报错 `Future is not Send`
- **根因**: tokio::Mutex guard 不是 `Send`，不能跨 `.await` 点持有。Go 的 `defer smLock.Unlock()` 可以跨 goroutine 让步，Rust 不行。
- **解法**: `enum StateMachine` + `std::mem::replace`。把状态 move 出来，释放锁后再操作，处理完再 swap 回去。
- **重复次数**: 1

### R02: `anyhow::bail!` 未导入直接调用

- **首次日期**: 2026-05-11
- **症状**: 调用 `bail!(...)` 编译失败，提示找不到 macro
- **根因**: `bail!` 是 macro，必须显式 `use anyhow::bail;`。Go 的 `fmt.Errorf` 是标准库自动可用。
- **解法**: 文件顶部加 `use anyhow::bail;`
- **重复次数**: 1

### R03: tokio::spawn JoinHandle 未 await 导致 panic 静默吞

- **首次日期**: 2026-05-11
- **症状**: `tokio::spawn` 启动的 task 内部 panic 了但服务无感知、无日志、无告警
- **根因**: Go 的 goroutine panic 会崩溃整个进程（无法忽略）。Rust 的 `tokio::spawn` 返回 `JoinHandle`，如果不 await 或 abort，task 的 panic 被静默捕获，Handle 被 drop 时悄悄释放。
- **解法**: `JoinHandle` 必须被 `handle.await` 或 `handle.abort()`。所有 spawn 的 task 都要有明确的 handle 生命周期管理——要么存在结构体字段里等 Drop 时 abort，要么 select! 里 join。
- **重复次数**: 1

### R04: 测试中全局函数变量注入失败

- **首次日期**: 2026-05-11
- **症状**: 写单元测试时想 mock 函数行为，用 `static mut NEW_FUNC: fn(...)` 全局变量注入，编译报 unsafe 或运行时竞争
- **根因**: Go 惯用的 `var newFunc = realImpl; test: newFunc = fake` 是包级可变变量，Rust 的 static mut 必须 unsafe 且不是 Send/Sync 安全的。
- **解法**: trait + `Box<dyn Fn>` 注入。定义 trait 作为依赖接口，生产用真实实现，测试注入 mock。类型安全、零 unsafe。
- **重复次数**: 1

---

## infra→业务 转型陷阱

> 首次做业务代码，infra 脑（吞吐/延迟/系统边界）容易在微观编码层踩坑。此处预埋常见陷阱，首次犯→观察，再次犯→入册。

### T01: hotpath 上 premature `.collect()` 导致多余 alloc

- **首次日期**: 2026-05-12
- **症状**: `chains.iter().map(score).collect::<Vec<_>>()` 后再 `.max_by()`——分配整个 Vec 只为取极值
- **根因**: infra 代码习惯"先收集再处理"（channel/task 通用模式）。业务热路径上 iterator 是惰性的，collect 会强制立即分配堆内存
- **解法**: iterator chain 全程 lazy：`chains.iter().filter_map(score).max_by(cmp)`。只在需要 len/index/跨线程 send/多次迭代时才 collect。阈值：lazy chain <10μs CPU → 不 collect
- **Tier C 锚点**: [AXON_PHILOSOPHY.md C1 惰性求值](./AXON_PHILOSOPHY.md#c1-惰性求值-lazy-first)
- **重复次数**: 0 (首次观测)

### T02: 热路径 `.unwrap()` / `.expect()` → production panic

- **首次日期**: 2026-05-12
- **症状**: 业务 handler 里 `chain.score.partial_cmp(&other.score).unwrap()`——score 为 NaN 时 panic 吞掉整个 request
- **根因**: infra 代码可以 unwrap（启动时挂=fast fail），业务代码每个 request 独立——一个 request 的 NaN 不应 kill 服务
- **解法**: 全部换 `?` + `AppError` 传播。`Option::ok_or(AppError::NoRoute)?` / `Result::map_err(|e| AppError::Internal(e.to_string()))?`。唯一的 unwrap 允许在 `main()` 启动阶段
- **Tier C 锚点**: [AXON_PHILOSOPHY.md C2 链式调用](./AXON_PHILOSOPHY.md#c2-链式调用-fluent-pipeline) — "禁 unwrap/expect 在生产路径"
- **重复次数**: 0 (首次观测)

### T03: 第一版就画 trait 图 → 抽象先行

- **首次日期**: 2026-05-12
- **症状**: 还没写 3 个 concrete func 就开始定义 `trait ChainRouter` / `trait Signer` / generic bounded `<T: RouteScore>`
- **根因**: infra 架构思维——系统设计从接口开始。但业务代码的抽象边界在第一版时是模糊的，第一版就画 trait = 猜错了就得全拆
- **解法**: Rule of Three — 3+ 处重复代码 → trait。1-2 处重复 → 保留 concrete func，继续观测。测试覆盖 >90% 才上泛型。
- **Tier C 锚点**: [AXON_PHILOSOPHY.md C4 必要抽象](./AXON_PHILOSOPHY.md#c4-必要抽象-rule-of-three)
- **重复次数**: 0 (首次观测)

---

## kp / KubePivot 工具链

> 暂无记录。首次犯→观察，再次犯→入册。

---

## Kubernetes / Helm

> 暂无记录。首次犯→观察，再次犯→入册。

---

## Docker / 镜像构建

> 暂无记录。首次犯→观察，再次犯→入册。

---

## 工程流程

### P01: 未按 commits/README.md 规范写 commit message

- **首次日期**: 2026-05-12
- **症状**: commit message 未遵循 conventional commits 格式（`type(scope): description`），加了 Co-Authored-By 伪标签，以及非 ASCII 字符在 subject 行
- **根因**: 未先读 `commits/README.md` 中的 KubePivot 提交规范（type 十选一、scope 仅 [a-zA-Z0-9-]、description 不能以 `.` 结尾、subject 强烈建议全 ASCII）
- **解法**: commit 前先读 `commits/README.md`，message 写入 `commits/<file>.txt`，用 `git commit -F commits/<file>.txt` 提交
- **重复次数**: 1

---

*最后更新: 2026-05-12 | 维护者: Axon 团队*
