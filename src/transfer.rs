use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::error::AppError;
use crate::router::Router;

// ── Request ──────────────────────────────────────────────
#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub from: String,
    pub to: String,
    /// Human amount, e.g. "0.5"
    pub amount: String,
    /// Optional: "ethereum" | "polygon" | "arbitrum" | "optimism"
    pub chain_hint: Option<String>,
}

// ── Response ─────────────────────────────────────────────
#[derive(Debug, Serialize)]
pub struct TransferResponse {
    pub tx_id: String,
    pub chain: String,
    pub route_score: f64,
    pub gas_estimate: String,
    pub status: String,
}

// ── Handler ──────────────────────────────────────────────
#[instrument(skip(router), fields(tx.from = req.from, tx.amount = req.amount))]
pub async fn transfer(
    State(router): State<Router>,
    Json(req): Json<TransferRequest>,
) -> Result<Json<TransferResponse>, AppError> {
    // 1. validate
    if req.from.is_empty() || req.to.is_empty() {
        return Err(AppError::InvalidTransfer("from/to required".into()));
    }
    if req.amount.parse::<f64>().unwrap_or(0.0) <= 0.0 {
        return Err(AppError::InvalidTransfer("amount must be > 0".into()));
    }

    // 2. route
    let chain = router
        .select(req.chain_hint.as_deref())
        .ok_or(AppError::NoRoute)?;

    // 3. sign (mock MPC — Sprint 2 替换 GG20)
    let tx_id = mock_sign(&req, chain.id);

    // 4. respond
    Ok(Json(TransferResponse {
        tx_id,
        chain: chain.name.into(),
        route_score: chain.score,
        gas_estimate: format!("{} gwei", chain.gas),
        status: "pending".into(),
    }))
}

fn mock_sign(req: &TransferRequest, chain_id: u64) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    req.from.hash(&mut h);
    req.to.hash(&mut h);
    req.amount.hash(&mut h);
    chain_id.hash(&mut h);
    format!("0x{:016x}{:016x}", h.finish(), h.finish())
}
