# Rust 学习笔记

> Axon 项目 Rust 讨论精华，从迭代器到底层机制。

---

## 一、迭代器

### 核心 trait

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

`next()` 是唯一的必须实现。其他方法（map/filter/fold 等）都是基于 `next()` 的默认实现。

### 三种创建方式

```rust
vec.iter()        // &T    — 不可变借用，不消费原集合
vec.iter_mut()    // &mut T — 可变借用
vec.into_iter()   // T     — 转移所有权，消费原集合
```

`next()` 返回值直接体现区别：

```rust
let v = vec![1, 2, 3];
let mut iter = v.iter();
assert_eq!(iter.next(), Some(&1));  // &i32，不是 i32
assert_eq!(iter.next(), Some(&2));
assert_eq!(iter.next(), Some(&3));
assert_eq!(iter.next(), None);
```

### 惰性求值

`map()`、`filter()` 等适配器不立刻执行——直到调用消耗型方法（`collect()`、`sum()`、`count()`）才拉数据。零中间分配。

```rust
// 适配器：链式声明，不执行
let v = vec![1, 2, 3, 4, 5];
let lazy = v.iter().map(|x| x * 2).filter(|x| *x > 5);

// 消耗型：触发执行
let result: Vec<i32> = lazy.collect();  // [6, 8, 10]
```

### 适配器 vs 消耗型

```
适配器（不消耗迭代器）             消耗型（消耗迭代器）
─────────────────────────────────────────────────
map / filter / take / skip / zip   collect / sum / product / count
enumerate / step_by / rev          find / nth / last / all / any
inspect / peekable / fuse          fold / for_each / max / min
```

### `for` 循环本质

`for` 循环底层就是迭代器。这俩等价：

```rust
for &num in vec.iter() { println!("{}", num); }

let mut iter = vec.iter();
while let Some(&num) = iter.next() { println!("{}", num); }
```

### 和闭包的关系

迭代器适配器天然配对闭包——`map(|x| ...)`、`filter(|&x| ...)`。闭包提供定制逻辑，迭代器提供遍历框架。

### 性能

惰性求值 + 编译器内联 + 无中间分配。Rust 迭代器的零成本抽象——展开后和手写循环一样的机器码。

---

## 二、解引用：谁动谁不动

### 算术运算符不自动解引用

```rust
let a = &5;
let b = &10;
// let c = a + b;  // ❌ &i32 + &i32 没实现
let c = *a + *b;    // ✅ 手动解引用
```

只有 `.` 方法调用走 auto-deref 链。`+` `-` `*` `/` 直接走 trait 匹配，不帮解。

但 `a % 2` 能编译——标准库给 `&i32` 显式实现了 `Rem<i32>`，不是编译器替你做了 auto-deref。

### for 循环不自动去引用

```rust
let v = vec![1, 2, 3];

for num in v.iter() {
    // num: &i32 — 引用还在！
}

for &num in v.iter() {
    // num: i32 — 模式匹配把 & 扒了一层
}
```

`println!("{}", num)` 不用 `*` 不是因为 for 解了引用，是 `Display` 给 `&T` 也实现了——println! 自己沿着 auto-deref 链找到了 `i32::fmt`。

### 引用和指针的界限

```
C++ 指针/引用：指针、引用、移动语义三种叠在一起
Rust：& = 借用，*mut T = 裸指针（unsafe），日常只碰 &
```

Rust 把模糊都替你管住了，`.` 自动解引用、模式匹配拆引用——不是界限变模糊，是活帮你干了。

---

## 三、引用的本质：`const T* const`

Rust 的 `&T` 既是"指针常量"也是"常量指针"——不能改指向，也不能通过它修改值：

```rust
let s1 = String::from("run");
let s2 = &s1;
// s2.push_str("oob");  // ❌ &T 禁止修改借用的值
```

```cpp
// C++ 引用 — T* const（指针常量）
int* const ref_like = &x;   // 指针本身不能变
*ref_like = 20;             // ✅ 可以通过它改值

// C++ const 引用 — const T* const
const int* const cref = &x; // 指针不能变，值也不能改

// Rust &T — 行为和 const T* const 一致
// Rust &mut T — 行为和 T* const 一致
```

C++ 里 `int&` 默认可改值，加 `const` 才禁。Rust 反过来——`&T` 默认不可改，加 `mut` 才放行。

---

## 四、闭包：带环境的函数

### 函数 vs 闭包

```rust
// 函数 — 只吃参数，不吃环境
fn add(a: i32, b: i32) -> i32 { a + b }

// 闭包 — 能捕获外部变量
let offset = 10;
let add_offset = |a, b| a + b + offset;  // 抓了 offset
```

普通函数 = 只认参数，不认环境。闭包 = 既认参数，又能抓走外部环境。

无捕获的闭包退化形式和匿名函数一模一样（`|a, b| a + b`），但它底层结构仍是闭包。

### C++ lambda 捕获 vs Rust

```
C++                   Rust
─────────────────────────────────────
[]  不捕获             不捕获（纯函数）
[&]  引用捕获          编译器按最小权限自动选借用
[=]  值捕获            move 闭包，但不是简单拷贝
```

Rust 不写 `[&]` `[=]`，编译器看你怎么用变量，自动按最小权限推导。

### Rust 三种捕获模式

```rust
// Fn — 只读借用（最小权限）
let s = String::from("hi");
let f = || println!("{}", s);
f(); f(); // 可以多次调用

// FnMut — 可变借用
let mut v = vec![1];
let mut g = || v.push(2);
g(); g(); // 也可以多次调用

// FnOnce — 拿走所有权（只能调一次）
let s2 = String::from("bye");
let h = || drop(s2);
// h(); // 只能调一次
```

### `move` ≠ C++ `[=]`

```rust
// C++ [=] — 拷贝副本，外面的 a 还在
// auto f = [=]() { return a; };

// Rust move — 转移所有权，外面不能再用了
let s = String::from("hi");
let f = move || println!("{}", s);
// println!("{}", s);  ← 编译错误
```

`Copy` 类型效果相同（都拷一份），非 `Copy` 类型 C++ 继续拷贝，Rust 直接搬走。

### `|&x|` 模式匹配

```rust
// iter() 出 &i32，filter 再包一层引用 → &&i32
// |&x| 是模式匹配，扒掉外层 &，剩下 &i32
.filter(|&x| x % 2 == 0)
```

`|&x|` 就是为了少写一层引用。它不是语法糖——它真的在匹配 `&&i32` 的形状。

### 返回闭包：`impl Fn` vs `Box<dyn Fn>`

```rust
// impl Fn — 静态分发，零运行时开销
fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
    move |y| x + y
}

// Box<dyn Fn> — 动态分发，运行时走 vtable
fn make_adder_boxed(x: i32) -> Box<dyn Fn(i32) -> i32> {
    Box::new(move |y| x + y)
}
```

```
impl Fn(i32) -> i32              Box<dyn Fn(i32)> -> i32
─────────────────────────────────────────────────────────
静态分发，编译时单态化               动态分发，运行时 vtable
零运行时开销                        堆分配 + 8 字节跳转
返回的闭包类型在编译期确定              类型擦除，可以返回不同类型
不能存多种闭包                       同签名闭包随意存
二进制膨胀（每调用点一份代码）              二进制不膨胀
```

**什么时候用哪个：**

```rust
// 99% 的场景：返回单一闭包，零开销
fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
    move |y| x + y
}

// 需要运行时选择不同闭包时才用 Box<dyn>
fn select_adder(kind: &str, x: i32) -> Box<dyn Fn(i32) -> i32> {
    match kind {
        "add" => Box::new(move |y| x + y),
        "mul" => Box::new(move |y| x * y),
        _     => Box::new(move |y| y),
        // 不同闭包 → 不同类型 → 必须擦除
    }
}
```

`impl Trait` 在返回值位置只是语法糖——编译器生成一个具体的匿名类型。需要运行时差异化返回时，只有 `Box<dyn>` 能接。

### 闭包和多线程

闭包天然适合多线程——捕获所需数据，`move` 转移所有权进线程：

```rust
use std::thread;

let nums = vec![1, 2, 3, 4, 5];
let handles = nums.into_iter().map(|num| {
    thread::spawn(move || {
        num * 2  // num 所有权被移进线程
    })
}).collect::<Vec<_>>();

for handle in handles {
    let result = handle.join().unwrap();
    println!("Result: {}", result);
}
```

`move` 在这里是关键——不 move，闭包捕获的是 `num` 的引用，活不过 `thread::spawn`。

### 闭包和性能

闭包是零成本抽象。Rust 编译器会为每个闭包生成独立的匿名类型，调用时直接内联——机器码和手写函数一致，没有虚调用开销（除非你显式用 `Box<dyn Fn>`）。

### 闭包和生命周期

闭包捕获的变量受生命周期系统保护——闭包不会比它捕获的任何变量活得更长。这是编译器静态保证的：悬垂闭包连编译都过不了，不需要运行时检查。

---

## 五、String 和 `&str`

概念上和 C++ 的 `std::string` / `const char*` 一一对应：

```
Rust            C++
────────────────────────────
String     →    std::string   拥有数据，堆分配，可扩容
&str       →    const char*   不拥有数据，只是借用/指向
```

但三个关键差异：

### 1. `&str` 是胖指针，`const char*` 是瘦指针

`&str` 内部存了 `(指针, 长度)`，不需要 `\0` 终止符——`str.len()` 是 O(1)。`const char*` 只有指针，靠 `\0` 判断结尾，`strlen` 是 O(n)。

### 2. `&str` 保证合法 UTF-8，`const char*` 不保证

```rust
let s = String::from("🦀");  // 4 bytes, 1 char, 编译器确保合法 UTF-8
// 而 const char* 可以是 GBK / Latin1 / 任何编码
```

### 3. `&str` 有生命周期，`const char*` 是裸指针

```cpp
// C++ — 悬垂指针，编译器不报
const char* p;
{
    std::string s = "hi";
    p = s.c_str();
}
puts(p);  // 💥 use-after-free
```

```rust
// Rust — 编译器拒绝
let p: &str;
{
    let s = String::from("hi");
    p = &s;
}
// println!("{}", p);  // ❌ s 活得不够长
```

**总结**：所有权维度一样，类型系统维度完全不同。这个类比适合快速建立直觉，但不适合用来推理 Rust 的借用规则。

---

## 六、数组 vs Vec

### `[T; N]` 和 `Vec<T>` 的底层差别

```
[T; N]（数组）           Vec<T>
──────────────────────────────────────────
栈上分配，大小编译期确定       堆上分配，可扩容运行时
大小是类型的一部分            大小是运行时值
[1; 100] 栈上放              100 个元素 = 堆分配
```

### 切片是统一抽象层

`&[T]` 是数组和 Vec 的**共同抽象**——类似 C++20 `std::span`：

```rust
let arr = [1, 3, 5, 7, 9];
let v = vec![1, 3, 5, 7, 9];

let part_arr = &arr[0..3];  // &[i32]
let part_vec = &v[0..3];    // &[i32] — 同一种类型
```

这两行得到的切片类型完全一样，`.iter()`、`.len()` 行为一致。所以函数参数写 `&[T]`，数组和 Vec 都能传。

### `into_iter()` 有坑

```rust
let arr = [1, 2, 3];
let v = vec![1, 2, 3];

// arr.into_iter() — 不会消费数组，迭代 &i32
// 因为 [i32; 3] 没有实现 IntoIterator，编译器退而求其次用迭代引用
for i in arr.into_iter() { println!("{}", i); }
println!("{:?}", arr); // ✅ 还能用

// v.into_iter() — 消费 Vec，迭代 i32
for i in v.into_iter() { println!("{}", i); }
// println!("{:?}", v); // ❌ v 已经被消费
```

根源：数组用 `.iter()` 或 `&arr` 迭代引用，Vec 用 `.into_iter()` 拿走所有权。显式写法最安全——想迭代就 `.iter()`，想消费就 `.into_iter()`，数组和 Vec 行为一致：

```rust
for i in arr.iter() { /* &i32 */ }   // 不消费
for i in v.iter() { /* &i32 */ }     // 不消费
for i in arr.into_iter() { /* i32 */ if arr: [i32;3] */ }  // 见上
for i in v.into_iter() { /* i32 */ } // Vec 被消费
```

### 什么时候用哪个

- 数量固定且编译期已知 → `[T; N]`，零堆分配
- 数量运行时确定 / 需要 grow / push → `Vec<T>`
- 函数参数只读不拥有 → `&[T]`，数组和 Vec 都能传

---

## 七、内存管理：RAII + 所有权

### 裸指针 vs shared_ptr

```cpp
// 问题：栈上的 shared_ptr 析构，裸指针悬垂
void addEvent() {
    shared_ptr<EventBase> eb = make_shared<EventBase>();
    mBase[fd] = eb.get();   // 悬垂——函数返回后 eb 析构
}

// 正确：owner 搬进容器
unordered_map<int, shared_ptr<EventBase>> mBase;  // 改这个
mBase[fd] = eb;  // 引用计数 +1，对象活到 erase 为止
```

原则：零 new/delete，全 RAII，裸指针只借不拥有。但 owner 必须活得比裸指针长——这不是 Rust 编译器会报的类型错误，在 C++ 里只能靠自己。

### SPSC 无锁队列

单生产者单消费者（SPSC）：同一个队列只有一个线程写、一个线程读。`eventfd` 的 `write`/`read` 提供内存屏障——`push` 发生在 `write` 之前，`read` 发生在 `swap` 之前。x86 上顺序有保证，不需要额外锁。但 C++ 标准层面没有线程安全语义，依赖的是实践保证。

---

## 八、Option 与 null：不存在的值不该存在

### 哲学

Rust 没有 null。不是删掉了，是从类型系统层面把"空"变成了一个可检查的状态。

```
C++                            Rust
────────────────────────────────────────────────
int* p = nullptr;               Option<&T>  — 不存在的引用
std::optional<T> (C++17)        Option<T>   — 不存在的值
返回 -1 / nullptr 表示错误        Result<T, E> — 用类型承载错误
```

Axon 对 null 的态度：**null 不在语言里，Option 在类型里。不存在的东西，编译器帮你记着。**

### 基本用法

```rust
// Some / None
let x: Option<i32> = Some(42);
let y: Option<i32> = None;

// match 穷尽
match x {
    Some(v) => println!("有值: {}", v),
    None => println!("空"),
}

// if let — 只关心有值的情况
if let Some(v) = x {
    println!("有值: {}", v);
}
```

### 常用方法链

```rust
// unwrap — 开发调试用，None 会 panic
let v = Some(42).unwrap();  // 42

// unwrap_or — 给默认值
let v = None.unwrap_or(0);  // 0

// map — 有值才变换
let len = Some("hi").map(|s| s.len());  // Some(2)
let len = None.map(|s: &str| s.len());  // None

// and_then — 链式处理，任一步 None 则短路
fn parse(s: &str) -> Option<i32> { s.parse().ok() }
let result = Some("42").and_then(parse);  // Some(42)
let result = None.and_then(parse);        // None
```

### 和 Result 的关系

Option 是"可能有也可能没有"，Result 是"要么对要么错"。前者用于值缺失，后者用于操作失败。两者可以互转：

```rust
// Option → Result: 给 None 一个错误
let v = opt.ok_or(anyhow::anyhow!("缺失必填字段"))?;

// Result → Option: 丢掉错误信息
let v = result.ok();  // Err → None
```

### Axon 的约定

```
函数返回可选值      → Option<T>，不用 null / -1 / 空字符串
函数可能失败        → Result<T, E>，不用异常
Option 链式处理      → map / and_then / unwrap_or，不写 match 套 match
unwrap / expect     → 仅用于"不应该为 None"的断言，不在业务逻辑用
```

一个你 C++ optional 实现里写了但 Rust 标准库已经做好的——`and_then`、`transform`（Rust 叫 `map`）、`or_else`、值判等——在 Rust 里都是 `Option<T>` 自带。不需要自己造。

---

## 九、环境变量存密钥

环境变量不是绝对安全——是工程上性价比最高的方案：

```
方案          不进 Git  不落文件  和代码解耦  部署隔离
─────────────────────────────────────────────
硬编码         ✗        ✗        ✗        ✗
配置文件        ✓        ✗        ✓        ✓
环境变量        ✓        ✓        ✓        ✓
Vault/Secrets  ✓        ✓        ✓        ✓（再加密一层）
```

防的是源码泄露、Git 提交暴露、明文文件被扒。不防 root 权限被拿、进程内存被 dump。环境变量是地板，不是天花板——往上走有 Vault、Sealed Secrets、云平台 Secret Manager。

---

*记于 Axon 开发中，从 C++ 到 Rust 的关键跳跃。*
