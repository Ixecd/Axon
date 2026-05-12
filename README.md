# Axon Protocol

> **Feelings 链上薪酬支付引擎** — 管发不管花。

---

## 这是什么

Axon 是 Feelings 项目的链上薪酬支付层。它做一件事：把工资/分成从 Feelings 财务地址发到员工和创作者的钱包地址。到账即止，之后的事一概不管。

## 起源

Axon 的出发点很简单：**链上支付不该这么难用。** 作者被币圈的反人类设计折磨够了——助记词、Gas、选链、Swap 跨 3 个 DApp——这些不该是用户操心的事。

最初想做通用钱包，后来发现 **把一件事做到极致胜过面面俱到的平庸**。这件事就是给 Feelings 发工资。当 scope 收窄到单一场景，极简才不是口号：两个 API，一条边界（到账止步），零多余概念。

```
通用钱包                          Axon Payroll
─────────────────────────────────────────────
助记词恐惧                         员工用 empId
先买 ETH 才付 Gas                  Paymaster 代付
选链选到焦虑                       Router 自动选
Swap 跨 3 个 DApp                  不存在（管发不管换）
关注用户消费体验                   关注薪酬合规发放
匿名地址即身份                     实名 → 地址绑定
```

**初心没变**——干掉币圈反人类设计。只是证明的方式从「做所有事」变成了「只做一件事，做到干净」。

## 核心流程

```
Feelings 财务
      │
      ▼
  POST /api/v1/disburse
      │
      ├─ ① resolve(empId) → 钱包地址  (身份层)
      ├─ ② router.select(chain)        (Route)
      ├─ ③ MPC sign + Paymaster gas    (签名层)
      └─ ④ tx broadcast → 链上到账     (结算层)
```

## API

```
# 员工身份绑定 (用工合同 → 钱包地址)
POST /api/v1/bind
  { "empId": "001", "name": "张三", "address": "0x...", "contractHash": "Qm..." }
  → { "empId": "001", "address": "0x...", "status": "bound" }

# 工资发放 (身份解析 + 路由 + gas代付)
POST /api/v1/disburse
  { "fromTreasury": "0x...", "empId": "001", "amount": "5000.00", "payrollBatchId": "2026-05" }
  → { "txHash": "0x...", "empAddress": "0x...", "chain": "polygon", "auditUrl": "ipfs://..." }
```

## 架构

```
Feelings (企业)
├── Treasury (财务地址)
│
Axon (本仓库)
├── Identity Registry   — empId → 地址绑定
├── Router              — 最优链选择 (Gas cheapest)
├── MPC Signer          — 企业级签名
├── Paymaster           — Gas 代付 (员工零 Gas)
└── Bridge              — 跨链 disbursement
```

## 边界

```
Axon 管的                   Axon 不管的
─────────────────────────────────────────
身份绑定 (empId → 地址)       员工怎么花钱
工资到账可审计                消费追踪
Gas 代付                     资产兑换
链上透明                     税务申报 (链上记录即证明)
```

## License

MIT License.

> Made by Ixecd, for Feelings employees and creators.
