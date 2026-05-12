// Axon Protocol — Feelings Payroll Hub
//
// MVP: identity binding + payroll disbursement
// Boundary: 工资到账止步，不追踪消费

pub mod error;
pub mod identity;
pub mod payroll;
pub mod router;

// Removed: pub mod transfer (→ payroll)
