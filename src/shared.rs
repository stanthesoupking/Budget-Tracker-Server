use serde::{Deserialize, Serialize};

use crate::budget::*;
use crate::budget_period::*;
use crate::transaction::*;

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
    pub email: String,
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
    pub email: String,
    pub first_name: String,
    pub last_name: String,
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
    pub budget_spend_limit: f64,
    pub budget_period_length: i64,
    pub budget_start_date: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanAccessBudgetForm {
    pub access_token: String,
    pub budget_id: i64,
    pub email: String
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionListResult {
    pub status: ResultStatus,
    pub transactions: Option<Vec<Transaction>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddTransactionForm {
    pub access_token: String,
    pub budget_id: i64,
    pub transaction_name: String,
    pub transaction_description: String,
    pub transaction_amount: f64,
    pub transaction_recur_days: Option<i64>,
    pub transaction_recur_until: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResult {
    pub status: ResultStatus,
    pub transaction: Option<Transaction>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetPeriodListResult {
    pub status: ResultStatus,
    pub budget_periods: Option<Vec<BudgetPeriod>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetPeriodForm {
    pub access_token: String,
    pub budget_id: i64,
    pub period_id: i64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetBalanceResult {
    pub status: ResultStatus,
    pub spent: Option<f64>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetPeriodResult {
    pub status: ResultStatus,
    pub budget_period: Option<BudgetPeriod>
}