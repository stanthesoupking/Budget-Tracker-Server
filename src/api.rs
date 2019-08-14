use actix_web::{web, Responder, Scope};

use crate::budget::Budget;
use crate::database::{User};
use crate::shared::*;

use crate::AppState;

pub fn get_service() -> Scope {
    web::scope("/api")
        .route("/register_user", web::post().to(register_user))
        .route("/get_access_token", web::post().to(get_access_token))
        .route("/change_password", web::post().to(change_password))
        .route("/list/budgets", web::post().to(list_budgets))
        .route("/add/budget", web::post().to(add_budget))
        .route("/delete/budget", web::post().to(delete_budget))
        .route("/get/budget", web::post().to(get_budget))
        .route("/list/can_access_budget", web::post().to(list_can_access_budget))
        .route("/add/can_access_budget", web::post().to(add_can_access_budget))
        .route("/delete/can_access_budget", web::post().to(delete_can_access_budget))
}

// API Routes

fn register_user(data: web::Data<AppState>, json: web::Json<CredentialForm>) -> impl Responder {
    let database = data.database.lock().unwrap();

    let user = User::new(&database, &json.username, &json.password, false);

    match database.insert_user(&user) {
        Ok(_) => web::Json(AccessTokenResult {
            status: ResultStatus::Success,
            access_token: Some(user.access_token),
        }),
        Err(error) => web::Json(AccessTokenResult {
            status: ResultStatus::Error(String::from(format!(
                "Error occurred while registering user: {:?}",
                error
            ))),
            access_token: None,
        }),
    }
}

fn get_access_token(data: web::Data<AppState>, json: web::Json<CredentialForm>) -> impl Responder {
    let database = data.database.lock().unwrap();

    let user = database.get_user_by_username(&json.username);

    match user {
        Ok(user) => match user {
            Some(user) => {
                if user.password == database.hash(&json.password) {
                    web::Json(AccessTokenResult {
                        status: ResultStatus::Success,
                        access_token: Some(user.access_token),
                    })
                } else {
                    web::Json(AccessTokenResult {
                        status: ResultStatus::InvalidCredentials,
                        access_token: None,
                    })
                }
            }
            None => web::Json(AccessTokenResult {
                status: ResultStatus::InvalidCredentials,
                access_token: None,
            }),
        },
        Err(error) => web::Json(AccessTokenResult {
            status: ResultStatus::Error(String::from(format!(
                "Error occurred while getting user access token: {:?}",
                error
            ))),
            access_token: None,
        }),
    }
}

fn change_password(
    data: web::Data<AppState>,
    json: web::Json<ChangePasswordForm>,
) -> impl Responder {
    let database = data.database.lock().unwrap();

    let user = database.get_user_by_access_token(&json.access_token);

    match user {
        Ok(user) => match user {
            Some(mut user) => {
                if user.password == database.hash(&json.current_password) {
                    // Change password + update access token
                    user.change_password(&database, &database.hash(&json.new_password));

                    match database.update_user(&user) {
                        Ok(_) => web::Json(AccessTokenResult {
                            status: ResultStatus::Success,
                            access_token: Some(user.access_token),
                        }),
                        Err(error) => web::Json(AccessTokenResult {
                            status: ResultStatus::Error(format!("Failed updating password: {:?}", error)),
                            access_token: None,
                        }),
                    }
                } else {
                    web::Json(AccessTokenResult {
                        status: ResultStatus::InvalidCredentials,
                        access_token: None,
                    })
                }
            }
            None => web::Json(AccessTokenResult {
                status: ResultStatus::InvalidAccessToken,
                access_token: None,
            }),
        },
        Err(error) => web::Json(AccessTokenResult {
            status: ResultStatus::Error(String::from(format!(
                "Error occurred while getting user access token: {:?}",
                error
            ))),
            access_token: None,
        }),
    }
}

fn list_budgets(data: web::Data<AppState>, json: web::Json<AccessTokenForm>) -> impl Responder {
    let database = data.database.lock().unwrap();

    let budgets = database.get_available_budgets(&json.access_token);

    match budgets {
        Ok(budgets) => web::Json(BudgetListResult {
            status: ResultStatus::Success,
            budgets: Some(budgets),
        }),
        Err(error) => web::Json(BudgetListResult {
            status: ResultStatus::Error(String::from(format!(
                "Error occurred while getting budgets: {:?}",
                error
            ))),
            budgets: None,
        }),
    }
}

fn add_budget(data: web::Data<AppState>, json: web::Json<AddBudgetForm>) -> impl Responder {
    let database = data.database.lock().unwrap();

    let budget = Budget::new(json.budget_name.clone(), json.budget_spend_limit, json.budget_period_length);

    let res = database.add_budget(&json.access_token, &budget);

    match res {
        Ok(budget) => web::Json(BudgetResult {
            status: ResultStatus::Success,
            budget: Some(budget),
        }),
        Err(error) => web::Json(BudgetResult {
            status: ResultStatus::Error(String::from(format!(
                "Error occurred creating budget: {:?}",
                error
            ))),
            budget: None,
        }),
    }
}

fn delete_budget(data: web::Data<AppState>, json: web::Json<SelectForm>) -> impl Responder {
    let database = data.database.lock().unwrap();

    let res = database.delete_budget(&json.access_token, json.id);

    match res {
        Ok(_) => web::Json(StatusResult {
            status: ResultStatus::Success,
        }),
        Err(error) => web::Json(StatusResult {
            status: ResultStatus::Error(String::from(format!(
                "Error occurred deleting budget: {:?}",
                error
            )))
        }),
    }
}

fn get_budget(data: web::Data<AppState>, json: web::Json<SelectForm>) -> impl Responder {
    let database = data.database.lock().unwrap();

    let res = database.get_available_budget(&json.access_token, json.id);

    match res {
        Ok(budget) => web::Json(BudgetResult {
            status: ResultStatus::Success,
            budget
        }),
        Err(error) => web::Json(BudgetResult {
            status: ResultStatus::Error(String::from(format!(
                "Error occurred getting budget: {:?}",
                error
            ))),
            budget: None
        }),
    }
}

fn list_can_access_budget(data: web::Data<AppState>, json: web::Json<SelectForm>) -> impl Responder {
    let database = data.database.lock().unwrap();

    let usernames = database.get_available_can_access_budget_users(&json.access_token, json.id);

    match usernames {
        Ok(usernames) => web::Json(UserListResult {
            status: ResultStatus::Success,
            users: Some(usernames),
        }),
        Err(error) => web::Json(UserListResult {
            status: ResultStatus::Error(String::from(format!(
                "Error occurred while getting users that have access to the
                given budget: {:?}",
                error
            ))),
            users: None,
        }),
    }
}

fn add_can_access_budget(data: web::Data<AppState>, json: web::Json<CanAccessBudgetForm>) -> impl Responder {
    let database = data.database.lock().unwrap();

    let res = database.add_can_access_budget(&json.access_token, json.budget_id, &json.username);

    match res {
        Ok(_) => web::Json(StatusResult {
            status: ResultStatus::Success
        }),
        Err(error) => web::Json(StatusResult {
            status: ResultStatus::Error(String::from(format!(
                "Error occurred giving budget access: {:?}",
                error
            )))
        }),
    }
}

fn delete_can_access_budget(data: web::Data<AppState>, json: web::Json<CanAccessBudgetForm>) -> impl Responder {
    let database = data.database.lock().unwrap();

    let res = database.delete_can_access_budget(&json.access_token, json.budget_id, &json.username);

    match res {
        Ok(_) => web::Json(StatusResult {
            status: ResultStatus::Success
        }),
        Err(error) => web::Json(StatusResult {
            status: ResultStatus::Error(String::from(format!(
                "Error occurred giving budget access: {:?}",
                error
            )))
        }),
    }
}