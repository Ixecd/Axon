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

## 三、引用的本质：`T* const`

C++ 里的引用本质是指针常量，不是常量指针：

```cpp
// 引用 — 本质是这个
int* const ref_like = &x;   // 指针本身是常量，不能指向别处
*ref_like = 20;             // ✅ 指向的值可以改

// 常量指针 — 不是引用
const int* ptr = &x;        // 指针本身可以变
ptr = &y;                   // ✅
// *ptr = 30;               // ❌ 指向的值不能改
```

引用 = `T* const` 的语法糖。`const T&` = `const T* const`。

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

---

## 五、内存管理：RAII + 所有权

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

## 六、环境变量存密钥

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
