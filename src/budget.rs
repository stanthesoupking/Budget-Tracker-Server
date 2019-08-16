use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Budget {
    pub budget_id: Option<i64>,
    pub owner: Option<String>,
    pub name: String,
    pub spend_limit: f64,
    pub period_length: i64,
    pub start_date: String
}

impl Budget {
    pub fn new(name: String, spend_limit: f64, period_length: i64, start_date: String) -> Budget {
        Budget {
            budget_id: None,
            owner: None,
            name,
            spend_limit,
            period_length,
            start_date
        }
    }
}