use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::error::AppError;
use crate::identity::EmployeeRegistry;
use crate::router::Router;

// ── App state ────────────────────────────────────────────
#[derive(Clone)]
pub struct AppState {
    pub router: Router,
    pub registry: EmployeeRegistry,
}

impl AppState {
    pub fn new(router: Router, registry: EmployeeRegistry) -> Self {
        Self { router, registry }
    }
}

// ── Bind: employee identity → wallet ─────────────────────
#[derive(Debug, Deserialize)]
pub struct BindRequest {
    pub emp_id: String,
    pub name: String,
    pub address: String,
    pub contract_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BindResponse {
    pub emp_id: String,
    pub address: String,
    pub status: String,
}

#[instrument(skip(state), fields(emp_id = req.emp_id))]
pub async fn bind(
    State(state): State<AppState>,
    Json(req): Json<BindRequest>,
) -> Result<Json<BindResponse>, AppError> {
    let emp = crate::identity::Employee {
        emp_id: req.emp_id.clone(),
        name: req.name.clone(),
        address: req.address.clone(),
        contract_hash: req.contract_hash.clone(),
        active: true,
    };
    let bound = state.registry.bind(emp)?;
    tracing::info!(
        emp_id = bound.emp_id,
        address = bound.address,
        "identity bound"
    );
    Ok(Json(BindResponse {
        emp_id: bound.emp_id,
        address: bound.address,
        status: "bound".into(),
    }))
}

// ── Disburse: payroll delivery ───────────────────────────
#[derive(Debug, Deserialize)]
pub struct DisburseRequest {
    /// Feelings 财务地址 (treasury)
    pub from_treasury: String,
    /// Employee ID (身份层解析 → 钱包地址)
    pub emp_id: String,
    /// Human-readable amount, e.g. "5000.00"
    pub amount: String,
    /// Token symbol, default "USDC"
    #[serde(default = "default_token")]
    pub token: String,
    /// Optional: chain hint for routing
    pub chain_hint: Option<String>,
    /// Paymaster gas代付 (default true — payroll must not burden employee)
    #[serde(default = "default_gas_paid")]
    pub gas_paid: bool,
    /// Payroll batch ID for audit trail
    pub payroll_batch_id: Option<String>,
}

fn default_token() -> String {
    "USDC".into()
}
fn default_gas_paid() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct DisburseResponse {
    pub tx_hash: String,
    pub emp_id: String,
    pub emp_address: String,
    pub chain: String,
    pub amount: String,
    pub gas_estimate: String,
    pub audit_url: Option<String>,
}

#[instrument(skip(state), fields(emp_id = req.emp_id, amount = req.amount))]
pub async fn disburse(
    State(state): State<AppState>,
    Json(req): Json<DisburseRequest>,
) -> Result<Json<DisburseResponse>, AppError> {
    // 1. validate input
    if req.from_treasury.is_empty() || req.emp_id.is_empty() {
        return Err(AppError::InvalidDisburse(
            "from_treasury/emp_id required".into(),
        ));
    }
    if req.amount.parse::<f64>().unwrap_or(0.0) <= 0.0 {
        return Err(AppError::InvalidDisburse("amount must be > 0".into()));
    }

    // 2. resolve identity: emp_id → wallet address
    let emp = state.registry.resolve(&req.emp_id)?;

    // 3. route: select optimal chain for disbursement
    let chain = state
        .router
        .select(req.chain_hint.as_deref())
        .ok_or(AppError::NoRoute)?;

    // 4. sign (mock MPC — Sprint 2 替换 GG20)
    let tx_hash = mock_sign(
        &emp.address,
        req.from_treasury.as_str(),
        &req.amount,
        chain.id,
    );

    let audit_url = req
        .payroll_batch_id
        .as_ref()
        .map(|batch| format!("ipfs://payroll/{}/{}", batch, tx_hash));

    tracing::info!(
        emp_id = emp.emp_id,
        amount = req.amount,
        chain = chain.name,
        tx = tx_hash,
        "disburse complete"
    );

    Ok(Json(DisburseResponse {
        tx_hash,
        emp_id: emp.emp_id,
        emp_address: emp.address,
        chain: chain.name.into(),
        amount: req.amount,
        gas_estimate: format!("{} gwei (paid by Paymaster)", chain.gas),
        audit_url,
    }))
}

fn mock_sign(to: &str, from: &str, amount: &str, chain_id: u64) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    from.hash(&mut h);
    to.hash(&mut h);
    amount.hash(&mut h);
    chain_id.hash(&mut h);
    format!("0x{:016x}{:016x}", h.finish(), h.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::EmployeeRegistry;
    use crate::router::Router;

    fn test_state() -> AppState {
        let reg = EmployeeRegistry::new();
        // pre-bind test employee
        let _ = reg.bind(crate::identity::Employee {
            emp_id: "emp-001".into(),
            name: "测试员工".into(),
            address: "0xtest".into(),
            contract_hash: Some("QmHash".into()),
            active: true,
        });
        AppState::new(Router::new(), reg)
    }

    #[tokio::test]
    async fn disburse_to_bound_employee() {
        let state = test_state();
        let req = Json(DisburseRequest {
            from_treasury: "0xtreasury".into(),
            emp_id: "emp-001".into(),
            amount: "5000.00".into(),
            token: "USDC".into(),
            chain_hint: None,
            gas_paid: true,
            payroll_batch_id: Some("2026-05-batch-01".into()),
        });
        let resp = disburse(State(state), req).await.unwrap();
        let body = resp.0;
        assert_eq!(body.emp_id, "emp-001");
        assert_eq!(body.amount, "5000.00");
        assert!(body.tx_hash.starts_with("0x"));
        assert!(body.audit_url.is_some());
    }

    #[tokio::test]
    async fn disburse_unbound_employee_fails() {
        let reg = EmployeeRegistry::new();
        let state = AppState::new(Router::new(), reg);
        let req = Json(DisburseRequest {
            from_treasury: "0xtreasury".into(),
            emp_id: "unknown".into(),
            amount: "1000.00".into(),
            token: "USDC".into(),
            chain_hint: None,
            gas_paid: true,
            payroll_batch_id: None,
        });
        let err = disburse(State(state), req).await.unwrap_err();
        assert!(matches!(err, AppError::InvalidIdentity(_)));
    }

    #[tokio::test]
    async fn disburse_zero_amount_fails() {
        let state = test_state();
        let req = Json(DisburseRequest {
            from_treasury: "0xtreasury".into(),
            emp_id: "emp-001".into(),
            amount: "0".into(),
            token: "USDC".into(),
            chain_hint: None,
            gas_paid: true,
            payroll_batch_id: None,
        });
        let err = disburse(State(state), req).await.unwrap_err();
        assert!(matches!(err, AppError::InvalidDisburse(_)));
    }
}
