// Identity Registry — Employee identity → wallet address mapping
//
// MVP (方案B): 雇佣关系绑定 — empId + 合同哈希 → 地址
// 升级A: 自然人KYC (税务>500w触发), 零知识证明Semaphore

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub emp_id: String,
    /// 实名 (姓名)
    pub name: String,
    /// 钱包地址 (EOA or AA contract)
    pub address: String,
    /// 劳动合同哈希 (IPFS or 链上)
    pub contract_hash: Option<String>,
    /// 雇佣状态
    pub active: bool,
}

#[derive(Debug, Clone, Default)]
pub struct EmployeeRegistry {
    employees: Arc<RwLock<HashMap<String, Employee>>>,
}

impl EmployeeRegistry {
    pub fn new() -> Self {
        Self {
            employees: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Bind an employee identity to a wallet address.
    /// Only active employees can receive payroll disbursements.
    pub fn bind(&self, emp: Employee) -> Result<Employee, AppError> {
        if emp.emp_id.is_empty() || emp.name.is_empty() || emp.address.is_empty() {
            return Err(AppError::InvalidIdentity(
                "emp_id/name/address required".into(),
            ));
        }
        let mut map = self
            .employees
            .write()
            .map_err(|e| AppError::Internal(format!("registry lock poisoned: {}", e)))?;
        map.insert(emp.emp_id.clone(), emp.clone());
        Ok(emp)
    }

    /// Resolve emp_id → Employee (only if active)
    pub fn resolve(&self, emp_id: &str) -> Result<Employee, AppError> {
        let map = self
            .employees
            .read()
            .map_err(|e| AppError::Internal(format!("registry lock poisoned: {}", e)))?;
        match map.get(emp_id) {
            Some(emp) if emp.active => Ok(emp.clone()),
            Some(_) => Err(AppError::InvalidIdentity(format!(
                "employee {} is inactive",
                emp_id
            ))),
            None => Err(AppError::InvalidIdentity(format!(
                "employee {} not found",
                emp_id
            ))),
        }
    }

    pub fn is_bound(&self, emp_id: &str) -> bool {
        self.employees
            .read()
            .map(|m| m.contains_key(emp_id))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_and_resolve() {
        let reg = EmployeeRegistry::new();
        let emp = Employee {
            emp_id: "emp-001".into(),
            name: "张三".into(),
            address: "0xabc123".into(),
            contract_hash: Some("QmHash".into()),
            active: true,
        };
        reg.bind(emp).unwrap();
        let resolved = reg.resolve("emp-001").unwrap();
        assert_eq!(resolved.name, "张三");
        assert_eq!(resolved.address, "0xabc123");
    }

    #[test]
    fn resolve_inactive_fails() {
        let reg = EmployeeRegistry::new();
        let emp = Employee {
            emp_id: "emp-002".into(),
            name: "李四".into(),
            address: "0xdef456".into(),
            contract_hash: None,
            active: false,
        };
        reg.bind(emp).unwrap();
        assert!(reg.resolve("emp-002").is_err());
    }

    #[test]
    fn resolve_unknown_fails() {
        let reg = EmployeeRegistry::new();
        assert!(reg.resolve("nobody").is_err());
    }

    #[test]
    fn bind_missing_fields() {
        let reg = EmployeeRegistry::new();
        let emp = Employee {
            emp_id: "".into(),
            name: "".into(),
            address: "".into(),
            contract_hash: None,
            active: true,
        };
        assert!(reg.bind(emp).is_err());
    }
}
