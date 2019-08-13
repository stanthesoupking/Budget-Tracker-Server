use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Budget {
    pub budget_id: Option<i64>,
    pub owner: Option<String>,
    pub name: String,
    pub period_length: i64
}

impl Budget {
    pub fn new(name: String, period_length: i64) -> Budget {
        Budget {
            budget_id: None,
            owner: None,
            name,
            period_length
        }
    }
}