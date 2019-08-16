use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetPeriod {
    pub period_id: i64,
    pub start_date: String, // Note - these dates are inclusive
    pub end_date: String,
}

impl BudgetPeriod {
    pub fn new(period_id: i64, start_date: String, end_date: String) -> BudgetPeriod {
        BudgetPeriod {
            period_id,
            start_date,
            end_date
        }
    }
}