use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CanAccessBudget {
    pub budget_id: i64,
    pub user_id: i64
}

impl CanAccessBudget {
    pub fn new(budget_id: i64, user_id: i64) -> CanAccessBudget {
        CanAccessBudget {
            budget_id,
            user_id
        }
    }
}