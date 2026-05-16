# Axon 开发规范

> 性质：强制约定，不是建议
> 范围：Axon 仓库所有 Rust 代码、文档、提交
> 更新：随项目演进持续修订

---

## 一、空值处理

### 铁律

Rust 没有 null。Axon 不使用任何形式的 null 替代品（空字符串、-1、哨兵值）。所有"可能不存在"的语义用 `Option<T>` 承载。所有"可能失败"的语义用 `Result<T, E>` 承载。

```
✅ Option<T>   — 值可能缺失
✅ Result<T,E> — 操作可能失败
❌ "" / -1 / nullptr / null  — 不存在
```

### 链式优先

```rust
// ✅ 链式处理，清晰无嵌套
let fee = tx.output
    .and_then(|o| o.value())
    .map(|v| v as f64 / 1e8)
    .unwrap_or(0.0);

// ❌ match 套 match，读起来累
let fee = match tx.output {
    Some(o) => match o.value() {
        Some(v) => v as f64 / 1e8,
        None => 0.0,
    },
    None => 0.0,
};
```

### unwrap/expect 仅用于断言

```rust
// ✅ 程序不变量——这里不应该是 None
let cfg = config.lock().unwrap();

// ❌ 业务逻辑里用 unwrap
let addr = req.to_address.unwrap();  // 崩给用户看？
```

---

## 二、错误处理

### 不用 String 当错误

```rust
// ❌ 丢了类型信息，调用方没法 match
fn do_thing() -> Result<(), String> { ... }

// ✅ 用 anyhow（应用层）或 thiserror（库层）
fn do_thing() -> anyhow::Result<()> { ... }
```

### 不用裸 panic

```rust
// ❌ 崩了连上下文都没有
panic!("出错了");

// ✅ 用 Result 传播，让调用方决定
return Err(anyhow::anyhow!("解析 {} 失败: {}", addr, source));
```

### ? 运算符优先

```rust
// ✅ 简洁
let user = queries.get_user(id).await?;

// ❌ 啰嗦
let user = match queries.get_user(id).await {
    Ok(u) => u,
    Err(e) => return Err(e.into()),
};
```

---

## 三、日志

### slog 体系

Axon 用 `slog` 不用 `log`。结构化日志，键值对格式。

```rust
// ✅ 结构化
slog::info!("提币完成", "tx_id", tx_id, "amount", amount);

// ❌ 字符串拼接
println!("提币完成: {} amount: {}", tx_id, amount);
```

### 日志级别约定

```
ERROR   需要人工介入的故障（DB 挂了、RPC 断了）
WARN    异常但可自动恢复（重试成功、降级处理）
INFO    关键业务流程节点（提币广播、充值确认）
DEBUG   开发调试信息（请求体、中间状态），生产不开
```

---

## 四、命名

### 文件与模块

```
snake_case     — 文件名、模块名
                identity.rs / payroll.rs / router.rs
```

### 类型与函数

```
CamelCase      — struct / enum / trait
snake_case     — fn / let / const
SCREAMING      — 全局常量（仅 const，不用 static mut）
```

### 缩写

不缩写。`address` 不是 `addr`，`transaction` 不是 `tx`。唯一的例外：变量名上下文已经明确、且缩写比全称更可读时可以用（比如 `tx_id` 在区块链上下文中比 `transaction_identifier` 好读）。

---

## 五、API 设计

### tonic gRPC

Axon 协议层走 tonic，proto 文件在 `proto/` 目录。

```protobuf
// 服务名 PascalCase
service PayrollService {
    // RPC 方法 PascalCase
    rpc SendPayment(SendPaymentRequest) returns (SendPaymentResponse);
}

// 消息 PascalCase
message SendPaymentRequest {
    // 字段 snake_case
    string employee_id = 1;
    uint64 amount = 2;
}
```

### 函数签名

参数顺序：context → 核心输入 → 选项。

```rust
// ✅ 一眼看到这个函数要什么
async fn send_payment(
    ctx: &Context,
    employee_id: &str,
    amount: u64,
    opts: PaymentOptions,
) -> Result<TxHash>

// ❌ 核心输入淹没在选项里
async fn send_payment(opts: PaymentOptions, id: &str, a: u64) -> Result<TxHash>
```

---

## 六、测试

### 单元测试放同文件

```rust
// 文件末尾
#[cfg(test)]
mod tests {
    use super::*;
    // ...
}
```

### 命名

```
test_<函数名>_<场景>_<预期>
```

```rust
#[test]
fn test_parse_address_btc_mainnet_returns_valid() { ... }
#[test]
fn test_parse_address_invalid_returns_error() { ... }
```

### 不测外部依赖

单元测试不走网络、不连 DB、不碰 etcd。外部依赖走集成测试或 mock trait。

---

## 七、提交格式

```
feat: xxx      — 新功能
fix: xxx       — 修 bug
docs: xxx      — 文档
refactor: xxx  — 重构（行为不变）
chore: xxx     — 杂项（依赖更新、格式化）
test: xxx      — 测试
```

提交信息一行概括，需要细节用空行后 body。英文优先，中文可穿插但保持混合风格一致。

---

## 八、禁止事项

```
❌ unsafe 代码（除非有性能基准证明必要 + 独立审查）
❌ unwrap() 在非测试/非断言代码中
❌ expect() 带无意义 message（"should work"）
❌ 硬编码密钥、地址、密码
❌ 裸指针（*const T / *mut T）
❌ 全局可变状态（static mut）
❌ .clone() 满天飞（先想借用）
❌ println! / eprintln!（走 slog）
```

---

*规范不是镣铐，是让后续的人不用猜你当时在想什么。*
