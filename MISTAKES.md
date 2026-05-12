# Axon 工程错误日志

> 哲学：**犯了两次及以上的错误都应该记录下来。单次错误是偶然，重复错误是认知盲区——必须显式化才能消除。**
>
> 规则：重复次数 ≥ 2 才入册。首次犯→观察；再次犯→记录根因+解法，全团队显式共享。

---

## 目录
- [Rust 语言陷阱](#rust-语言陷阱)
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
