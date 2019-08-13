use serde::{Deserialize, Serialize};

use crate::budget::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum ResultStatus {
    Success,
    InvalidCredentials,
    InvalidAccessToken,
    EntryDoesNotExist,
    Error(String)
}

// --- FORMS ---

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialForm {
    pub username: String,
    pub password: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordForm {
    pub access_token: String,
    pub current_password: String,
    pub new_password: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterAccountForm {
    pub username: String,
    pub password: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenForm {
    pub access_token: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectForm {
    pub access_token: String,
    pub id: i64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddBudgetForm {
    pub access_token: String,
    pub budget_name: String,
    pub budget_period_length: i64
}

// --- RESULTS

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenResult {
    pub status: ResultStatus,
    pub access_token: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResult {
    pub status: ResultStatus
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetListResult {
    pub status: ResultStatus,
    pub budgets: Option<Vec<Budget>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetResult {
    pub status: ResultStatus,
    pub budget: Option<Budget>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserListResult {
    pub status: ResultStatus,
    pub users: Option<Vec<String>>
}


