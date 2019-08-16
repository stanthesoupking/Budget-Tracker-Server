use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_id: Option<i64>,
    pub budget_id: i64,
    pub email: Option<String>,
    pub name: String,
    pub description: String,
    pub date: Option<String>,
    pub amount: f64,
    pub recur_days: Option<i64>,
    pub recur_until: Option<String>
}

impl Transaction {
    pub fn new(budget_id: i64, name: String, description: String,
               amount: f64, recur_days: Option<i64>, recur_until: Option<String>) -> Transaction {
        Transaction {
            transaction_id: None,
            budget_id,
            email: None,
            name,
            description,
            date: None,
            amount,
            recur_days,
            recur_until
        }
    }
}