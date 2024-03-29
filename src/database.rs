use crate::budget::Budget;
use crate::budget_period::BudgetPeriod;
use crate::transaction::Transaction;
use crate::util::*;

use std::fs;
use std::io;
use std::path::Path;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

use rusqlite::Error::SqliteFailure;
use rusqlite::{params, Connection, NO_PARAMS};

use chrono::{DateTime, Duration, FixedOffset, Utc};

use time::Duration as OldDuration;

use termion::input::TermRead;

#[derive(Debug)]
pub enum Error {
    LoadFileError,
    CreateAdminError,
    EntryNotFound,
    UserAlreadyExists,
    UpdateEntryMissingID,
    InvalidCredentials,
    UserDeniedError,
    AccessRecursionError,
    SqliteError(libsqlite3_sys::Error, Option<String>),
    UnknownError,
}

impl std::convert::From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        println!("SQLITE ERROR: {:#?}", error);
        Error::UnknownError
    }
}

impl std::convert::From<chrono::ParseError> for Error {
    fn from(error: chrono::ParseError) -> Self {
        println!("TIME PARSING ERROR: {:#?}", error);
        Error::UnknownError
    }
}

pub struct Database {
    db_conn: Connection,
    secret: String,
}

impl Database {
    pub fn new(secret: String, path: &str) -> Result<Database, Error> {
        let rpath = Path::new(path);

        // Check if db file exist
        let init_req = !rpath.exists();

        // Connect to sqlite database
        let db_conn = match Connection::open(path) {
            Ok(conn) => conn,
            Err(_) => return Err(Error::LoadFileError),
        };

        // Enable foreign key support
        db_conn
            .execute("PRAGMA foreign_keys = ON", NO_PARAMS)
            .expect("Failed enabling foreign key support.");

        let database = Database { secret, db_conn };

        // Does the database need to be initialised?
        if init_req {
            match database.init() {
                Err(error) => rollback(path, error),
                Ok(_) => {
                    // Create admin user
                    println!(" === Admin User Setup ===");

                    println!("email:");

                    let mut buffer = String::new();

                    io::stdin()
                        .read_line(&mut buffer)
                        .expect("Failed reading line.");
                    let email = buffer;

                    let password: String;
                    loop {
                        println!("Password:");
                        let buffer = io::stdin().read_passwd(&mut io::stdout());

                        match buffer {
                            Ok(Some(p)) => {
                                password = p;
                                break;
                            }
                            _ => (),
                        }
                    }

                    io::stdin()
                        .read_passwd(&mut io::stdout())
                        .expect("Failed reading password");

                    let admin_user = User::new(
                        &database,
                        &email,
                        &String::from("Administrator"),
                        &String::from("Account"),
                        &password,
                        true,
                    );

                    match database.insert_user(&admin_user) {
                        Ok(_) => {
                            println!("Admin user created.");
                        }
                        Err(error) => {
                            println!("Error: Failed creating admin user.");
                            rollback(path, error)
                        }
                    };
                }
            };
        }

        Ok(database)
    }

    fn init(&self) -> Result<(), Error> {
        match self.db_conn.execute_batch(
            "
            CREATE TABLE users (
                email TEXT NOT NULL PRIMARY KEY,
                first_name TEXT NOT NULL,
                last_name TEXT NOT NULL,
                password TEXT NOT NULL,
                access_token TEXT NOT NULL,
                is_admin BOOL NOT NULL DEFAULT FALSE
            );

            CREATE TABLE budgets (
                budget_id INTEGER PRIMARY KEY AUTOINCREMENT,
                owner TEXT NOT NULL,
                name TEXT NOT NULL,
                spend_limit FLOAT NOT NULL,
                period_length INTEGER NOT NULL,
                start_date TEXT NOT NULL,
                FOREIGN KEY(owner) REFERENCES users(email)
            );

            CREATE TABLE can_access_budget (
                budget_id INTEGER NOT NULL,
                email TEXT NOT NULL,
                PRIMARY KEY(budget_id, email),
                FOREIGN KEY(budget_id) REFERENCES budgets(budget_id),
                FOREIGN KEY(email) REFERENCES users(email)
            );

            CREATE TABLE transactions (
                transaction_id INTEGER PRIMARY KEY AUTOINCREMENT,
                budget_id INTEGER NOT NULL,
                email TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                date TEXT NOT NULL,
                amount FLOAT NOT NULL,
                recur_days INTEGER NOT NULL,
                recur_until TEXT,
                FOREIGN KEY(budget_id) REFERENCES budgets(budget_id)
                FOREIGN KEY(email) REFERENCES users(email)
            );
            ",
        ) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::UnknownError),
        }
    }

    pub fn hash(&self, s: &String) -> String {
        let mut hasher = Sha256::new();

        // Add input string
        hasher.input_str(s.as_str());

        // Add secret
        hasher.input_str(self.secret.as_str());

        // Return hashed string
        hasher.result_str()
    }

    pub fn insert_user(&self, user: &User) -> Result<(), Error> {
        let res = self.db_conn.execute(
            "INSERT INTO users(
                email, first_name, last_name, password, access_token, is_admin
            )
            VALUES(?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                user.email,
                user.first_name,
                user.last_name,
                user.password,
                user.access_token,
                user.is_admin
            ],
        );

        match res {
            Ok(_) => Ok(()),
            Err(error) => match error {
                SqliteFailure(error, _) => match error.code {
                    rusqlite::ErrorCode::ConstraintViolation => Err(Error::UserAlreadyExists),
                    _ => Err(Error::UnknownError),
                },
                _ => Err(Error::UnknownError),
            },
        }
    }

    pub fn get_user_by_email(&self, email: &str) -> Result<Option<User>, Error> {
        let mut stmt = self.db_conn.prepare(
            "SELECT email, first_name, last_name, password, access_token, is_admin
            FROM users WHERE email = ?1",
        )?;

        match stmt.query_row(params![email], |row| {
            Ok(User {
                email: row.get(0)?,
                first_name: row.get(1)?,
                last_name: row.get(2)?,
                password: row.get(3)?,
                access_token: row.get(4)?,
                is_admin: row.get(5)?,
            })
        }) {
            Ok(user) => Ok(Some(user)),
            Err(error) => match error {
                QueryReturnedNoRows => Ok(None),
                _ => Err(Error::UnknownError),
            },
        }
    }

    pub fn get_user_by_access_token(&self, access_token: &str) -> Result<Option<User>, Error> {
        let mut stmt = self.db_conn.prepare(
            "SELECT email, first_name, last_name, password, access_token, is_admin
            FROM users WHERE access_token = ?1",
        )?;

        match stmt.query_row(params![access_token], |row| {
            Ok(User {
                email: row.get(0)?,
                first_name: row.get(1)?,
                last_name: row.get(2)?,
                password: row.get(3)?,
                access_token: row.get(4)?,
                is_admin: row.get(5)?,
            })
        }) {
            Ok(user) => Ok(Some(user)),
            Err(error) => match error {
                QueryReturnedNoRows => Ok(None),
                _ => Err(Error::UnknownError),
            },
        }
    }

    pub fn update_user(&self, user: &User) -> Result<(), Error> {
        let res = self.db_conn.execute(
            "UPDATE users SET first_name = ?1, SET last_name ?2,
            SET password = ?3, access_token = ?4, is_admin = ?5
            WHERE email = ?4",
            params![
                user.first_name,
                user.last_name,
                user.password,
                user.access_token,
                user.is_admin,
                user.email
            ],
        );

        match res {
            Ok(_) => Ok(()),
            Err(error) => match error {
                SqliteFailure(error, desc) => Err(Error::SqliteError(error, desc)),
                _ => Err(Error::UnknownError),
            },
        }
    }

    pub fn get_available_budgets(&self, access_token: &str) -> Result<Vec<Budget>, Error> {
        let mut stmt = self.db_conn.prepare(
            "SELECT budget_id, owner, name, spend_limit, period_length, start_date FROM budgets WHERE budget_id in (
            SELECT budget_id FROM (SELECT budget_id, owner AS email FROM budgets
            UNION SELECT budget_id, email FROM can_access_budget) WHERE email in
            (SELECT email FROM users WHERE access_token = ?1))"
        )?;

        let mut result: Vec<Budget> = Vec::new();

        let budget_iter = stmt.query_map(params![access_token], |row| {
            Ok(Budget {
                budget_id: row.get(0)?,
                owner: row.get(1)?,
                name: row.get(2)?,
                spend_limit: row.get(3)?,
                period_length: row.get(4)?,
                start_date: row.get(5)?,
            })
        });

        for budget in budget_iter? {
            result.push(budget?);
        }

        Ok(result)
    }

    pub fn add_budget(&self, access_token: &str, budget: &Budget) -> Result<Budget, Error> {
        let user = match self.get_user_by_access_token(access_token)? {
            Some(x) => x,
            None => return Err(Error::InvalidCredentials),
        };

        let res = self.db_conn.execute(
            "INSERT INTO budgets(
                owner, name, spend_limit, period_length, start_date
            )
            VALUES(?1, ?2, ?3, ?4, ?5)",
            params![
                user.email,
                budget.name,
                budget.spend_limit,
                budget.period_length,
                budget.start_date
            ],
        );

        match res {
            Ok(_) => {
                let budget_id = self.db_conn.last_insert_rowid();
                Ok(Budget {
                    budget_id: Some(budget_id),
                    owner: Some(user.email),
                    name: budget.name.clone(),
                    spend_limit: budget.spend_limit,
                    period_length: budget.period_length,
                    start_date: budget.start_date.clone(),
                })
            }
            Err(error) => match error {
                SqliteFailure(error, desc) => Err(Error::SqliteError(error, desc)),
                _ => Err(Error::UnknownError),
            },
        }
    }

    pub fn get_budget(&self, budget_id: i64) -> Result<Option<Budget>, Error> {
        // Get budget
        let mut stmt = self.db_conn.prepare(
            "SELECT budget_id, owner, name, spend_limit, period_length, start_date FROM budgets
            WHERE budget_id = ?1",
        )?;

        match stmt.query_row(params![budget_id], |row| {
            Ok(Budget {
                budget_id: row.get(0)?,
                owner: row.get(1)?,
                name: row.get(2)?,
                spend_limit: row.get(3)?,
                period_length: row.get(4)?,
                start_date: row.get(5)?,
            })
        }) {
            Ok(budget) => Ok(Some(budget)),
            Err(error) => match error {
                QueryReturnedNoRows => Ok(None),
                _ => Err(Error::UnknownError),
            },
        }
    }

    pub fn get_available_budget(
        &self,
        access_token: &str,
        budget_id: i64,
    ) -> Result<Option<Budget>, Error> {
        // Get available budget
        let mut stmt = self.db_conn.prepare(
            "SELECT budget_id, owner, name, spend_limit, period_length, start_date FROM budgets WHERE budget_id = ?1 AND budget_id in (
            SELECT budget_id FROM (SELECT budget_id, owner AS email FROM budgets
            UNION SELECT budget_id, email FROM can_access_budget) WHERE email in
            (SELECT email FROM users WHERE access_token = ?2))"
        )?;

        match stmt.query_row(params![budget_id, access_token], |row| {
            Ok(Budget {
                budget_id: row.get(0)?,
                owner: row.get(1)?,
                name: row.get(2)?,
                spend_limit: row.get(3)?,
                period_length: row.get(4)?,
                start_date: row.get(5)?,
            })
        }) {
            Ok(budget) => Ok(Some(budget)),
            Err(error) => match error {
                QueryReturnedNoRows => Ok(None),
                _ => Err(Error::UnknownError),
            },
        }
    }

    pub fn delete_budget(&self, access_token: &str, budget_id: i64) -> Result<(), Error> {
        let user = match self.get_user_by_access_token(access_token)? {
            Some(x) => x,
            None => return Err(Error::InvalidCredentials),
        };

        // Get budget
        let budget = self.get_budget(budget_id)?;

        match budget {
            Some(budget) => {
                let owner = match budget.owner {
                    Some(id) => id,
                    None => return Err(Error::UnknownError),
                };

                // Check if user is the budget owner
                if owner != user.email {
                    return Err(Error::UserDeniedError);
                }

                // Perform deletion
                let res = self.db_conn.execute(
                    "DELETE FROM budgets WHERE budget_id = ?1",
                    params![budget_id],
                );
                match res {
                    Ok(_) => Ok(()),
                    Err(error) => match error {
                        SqliteFailure(error, desc) => Err(Error::SqliteError(error, desc)),
                        _ => Err(Error::UnknownError),
                    },
                }
            }
            None => Err(Error::EntryNotFound),
        }
    }

    pub fn get_available_can_access_budget_users(
        &self,
        access_token: &str,
        budget_id: i64,
    ) -> Result<Vec<String>, Error> {
        let user = match self.get_user_by_access_token(access_token)? {
            Some(x) => x,
            None => return Err(Error::InvalidCredentials),
        };

        let mut stmt = self.db_conn.prepare(
            "SELECT email FROM users WHERE email IN (SELECT email FROM (SELECT owner AS email FROM budgets WHERE budget_id = ?1
            UNION SELECT email FROM can_access_budget WHERE budget_id = ?1))"
        )?;

        let mut is_available = false;
        let mut result: Vec<String> = Vec::new();

        let email_iter = stmt.query_map(params![budget_id], |row| Ok(row.get(0)?));

        for email in email_iter? {
            let email = email?;

            if email == user.email {
                is_available = true;
            }

            result.push(email);
        }

        if result.len() == 0 {
            Err(Error::EntryNotFound)
        } else if is_available {
            Ok(result)
        } else {
            Err(Error::UserDeniedError)
        }
    }

    pub fn add_can_access_budget(
        &self,
        access_token: &str,
        budget_id: i64,
        email: &str,
    ) -> Result<(), Error> {
        // Get current user
        let user = match self.get_user_by_access_token(access_token)? {
            Some(x) => x,
            None => return Err(Error::InvalidCredentials),
        };

        // Get budget
        let budget = match self.get_budget(budget_id) {
            Ok(budget) => match budget {
                Some(budget) => budget,
                None => return Err(Error::EntryNotFound),
            },
            Err(error) => return Err(error),
        };

        // Check if the current user is the budget owner
        match budget.owner {
            Some(owner) => {
                if !owner.eq(&user.email) {
                    return Err(Error::UserDeniedError);
                }
            }
            None => return Err(Error::UnknownError),
        };

        // Check if the request is trying to give owner access to their own budget
        if email.eq(&user.email) {
            return Err(Error::AccessRecursionError);
        }

        let res = self.db_conn.execute(
            "INSERT INTO can_access_budget(
                budget_id, email
            )
            VALUES(?1, ?2)",
            params![budget_id, email],
        );

        match res {
            Ok(_) => Ok(()),
            Err(error) => match error {
                SqliteFailure(error, desc) => Err(Error::SqliteError(error, desc)),
                _ => Err(Error::UnknownError),
            },
        }
    }

    pub fn delete_can_access_budget(
        &self,
        access_token: &str,
        budget_id: i64,
        email: &str,
    ) -> Result<(), Error> {
        let user = match self.get_user_by_access_token(access_token)? {
            Some(x) => x,
            None => return Err(Error::InvalidCredentials),
        };

        // Get budget
        let budget = self.get_budget(budget_id)?;

        match budget {
            Some(budget) => {
                let owner = match budget.owner {
                    Some(id) => id,
                    None => return Err(Error::UnknownError),
                };

                // Check if user is the budget owner
                if owner != user.email {
                    return Err(Error::UserDeniedError);
                }

                // Perform deletion
                let res = self.db_conn.execute(
                    "DELETE FROM can_access_budget WHERE budget_id = ?1 AND email = ?2",
                    params![budget_id, email],
                );
                match res {
                    Ok(_) => Ok(()),
                    Err(error) => match error {
                        SqliteFailure(error, desc) => Err(Error::SqliteError(error, desc)),
                        _ => Err(Error::UnknownError),
                    },
                }
            }
            None => Err(Error::EntryNotFound),
        }
    }

    pub fn get_budget_transactions(
        &self,
        access_token: &str,
        budget_id: i64,
    ) -> Result<Vec<Transaction>, Error> {
        let user = match self.get_user_by_access_token(access_token)? {
            Some(x) => x,
            None => return Err(Error::InvalidCredentials),
        };
        let budget = match self.get_available_budget(access_token, budget_id)? {
            Some(x) => x,
            None => return Err(Error::EntryNotFound),
        };

        let mut stmt = self.db_conn.prepare(
            "SELECT transaction_id, budget_id, email, name, description, date, amount, recur_days,
            recur_until FROM transactions WHERE budget_id = ?1 ORDER BY date DESC",
        )?;

        let mut result: Vec<Transaction> = Vec::new();

        let transaction_iter = stmt.query_map(params![budget_id], |row| {
            Ok(Transaction {
                transaction_id: row.get(0)?,
                budget_id: row.get(1)?,
                email: row.get(2)?,
                name: row.get(3)?,
                description: row.get(4)?,
                date: row.get(5)?,
                amount: row.get(6)?,
                recur_days: row.get(7)?,
                recur_until: row.get(8)?,
            })
        });

        for transaction in transaction_iter? {
            result.push(transaction?);
        }

        Ok(result)
    }

    pub fn get_budget_transactions_in_period(
        &self,
        access_token: &str,
        budget_id: i64,
        period_id: i64,
    ) -> Result<Vec<Transaction>, Error> {
        let user = match self.get_user_by_access_token(access_token)? {
            Some(x) => x,
            None => return Err(Error::InvalidCredentials),
        };
        let budget = match self.get_available_budget(access_token, budget_id)? {
            Some(x) => x,
            None => return Err(Error::EntryNotFound),
        };

        // Get period
        let period = match self.get_budget_period(access_token, budget_id, period_id)? {
            Some(x) => x,
            None => return Err(Error::EntryNotFound)
        };

        let mut stmt = self.db_conn.prepare(
            "SELECT transaction_id, budget_id, email, name, description, date, amount, recur_days,
            recur_until FROM transactions WHERE budget_id = ?1 AND date(date) >= date(?2) AND date(date) <= date(?3) ORDER BY date DESC",
        )?;

        let mut result: Vec<Transaction> = Vec::new();

        let transaction_iter = stmt.query_map(params![budget_id, period.start_date, period.end_date], |row| {
            Ok(Transaction {
                transaction_id: row.get(0)?,
                budget_id: row.get(1)?,
                email: row.get(2)?,
                name: row.get(3)?,
                description: row.get(4)?,
                date: row.get(5)?,
                amount: row.get(6)?,
                recur_days: row.get(7)?,
                recur_until: row.get(8)?,
            })
        });

        for transaction in transaction_iter? {
            result.push(transaction?);
        }

        Ok(result)
    }

    pub fn add_transaction(
        &self,
        access_token: &str,
        transaction: &Transaction,
    ) -> Result<Transaction, Error> {
        let user = match self.get_user_by_access_token(access_token)? {
            Some(x) => x,
            None => return Err(Error::InvalidCredentials),
        };

        let date = match &transaction.date {
            Some(x) => x.clone(),
            None => get_current_date_time(),
        };

        // TODO: verify that the current user has access to this budget

        let res = self.db_conn.execute(
            "INSERT INTO transactions(
                budget_id, email, name, description, date, amount, recur_days, recur_until
            )
            VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                transaction.budget_id,
                user.email,
                transaction.name,
                transaction.description,
                date,
                transaction.amount,
                transaction.recur_days,
                transaction.recur_until
            ],
        );

        match res {
            Ok(_) => {
                let transaction_id = self.db_conn.last_insert_rowid();
                Ok(Transaction {
                    transaction_id: Some(transaction_id),
                    budget_id: transaction.budget_id,
                    email: Some(user.email.clone()),
                    name: transaction.name.clone(),
                    description: transaction.description.clone(),
                    date: transaction.date.clone(),
                    amount: transaction.amount,
                    recur_days: transaction.recur_days,
                    recur_until: transaction.recur_until.clone(),
                })
            }
            Err(error) => match error {
                SqliteFailure(error, desc) => Err(Error::SqliteError(error, desc)),
                _ => Err(Error::UnknownError),
            },
        }
    }

    pub fn get_budget_periods(
        &self,
        access_token: &str,
        budget_id: i64,
    ) -> Result<Vec<BudgetPeriod>, Error> {
        let user = match self.get_user_by_access_token(access_token)? {
            Some(x) => x,
            None => return Err(Error::InvalidCredentials),
        };

        let budget = match self.get_available_budget(access_token, budget_id)? {
            Some(x) => x,
            None => return Err(Error::InvalidCredentials),
        };

        let mut start_date: DateTime<FixedOffset> = from_sqlite_date(&budget.start_date)?;

        let today: DateTime<FixedOffset> = DateTime::parse_from_rfc2822(&Utc::now().to_rfc2822())?;
        let mut period_id: i64 = 0;

        let mut res: Vec<BudgetPeriod> = Vec::new();
        loop {
            let end_date = start_date + Duration::days(budget.period_length - 1);
            let period = BudgetPeriod {
                period_id,
                start_date: to_sqlite_date(&start_date),
                end_date: to_sqlite_date(&end_date),
            };

            res.insert(0, period);

            start_date = end_date + Duration::days(1);

            if start_date > today {
                break;
            }

            period_id += 1;
        }

        Ok(res)
    }

    pub fn get_budget_period(
        &self,
        access_token: &str,
        budget_id: i64,
        period_id: i64
    ) -> Result<Option<BudgetPeriod>, Error> {
        let user = match self.get_user_by_access_token(access_token)? {
            Some(x) => x,
            None => return Err(Error::InvalidCredentials),
        };

        let budget = match self.get_available_budget(access_token, budget_id)? {
            Some(x) => x,
            None => return Err(Error::InvalidCredentials),
        };

        let mut start_date: DateTime<FixedOffset> = from_sqlite_date(&budget.start_date)?;

        start_date = start_date + Duration::days(budget.period_length * period_id);
        let end_date = start_date + Duration::days(budget.period_length -1);

        if start_date < from_sqlite_date(&budget.start_date)? {
            Ok(None)
        } else {
            Ok(Some(BudgetPeriod {
                period_id,
                start_date: to_sqlite_date(&start_date),
                end_date: to_sqlite_date(&end_date)
            }))
        }
    }

    pub fn get_budget_period_amount_spent(
        &self,
        access_token: &str,
        budget_id: i64,
        period_id: i64
    ) -> Result<f64, Error> {
        let user = match self.get_user_by_access_token(access_token)? {
            Some(x) => x,
            None => return Err(Error::InvalidCredentials),
        };

        let budget = match self.get_available_budget(access_token, budget_id)? {
            Some(x) => x,
            None => return Err(Error::InvalidCredentials),
        };

        let mut stmt = self.db_conn.prepare(
            "SELECT SUM(amount), substr(date, 0, 11) FROM transactions WHERE
            budget_id = ?1 AND
            date >= ?2 AND date <= ?3"
        )?;

        let period = match self.get_budget_period(access_token, budget_id, period_id)? {
            Some(x) => x,
            None => return Err(Error::EntryNotFound),
        };

        match stmt.query_row(params![budget_id, period.start_date, period.end_date], |row| {
            let amount: f64 = row.get(0)?;
            Ok(amount)
        }) {
            Ok(amount) => Ok(amount),
            Err(_) => Ok(0.0)
        }
    }

    pub fn get_current_budget_period(
        &self,
        access_token: &str,
        budget_id: i64
    ) -> Result<BudgetPeriod, Error> {
        let user = match self.get_user_by_access_token(access_token)? {
            Some(x) => x,
            None => return Err(Error::InvalidCredentials),
        };

        let budget = match self.get_available_budget(access_token, budget_id)? {
            Some(x) => x,
            None => return Err(Error::InvalidCredentials),
        };

        let start_date: DateTime<FixedOffset> = from_sqlite_date(&budget.start_date)?;

        let today: DateTime<FixedOffset> = get_now();

        let duration = today.signed_duration_since(start_date);

        let period_id = duration.num_days() / budget.period_length;

        let start_date = start_date + Duration::days(period_id * budget.period_length);
        let end_date = start_date + Duration::days(budget.period_length - 1);

        Ok(BudgetPeriod {
            period_id,
            start_date: to_sqlite_date(&start_date),
            end_date: to_sqlite_date(&end_date)
        })
    }
}

fn rollback(path: &str, error: Error) {
    // Do rollback
    println!("Error occurred while setting up database, rolling back changes...");
    fs::remove_file(Path::new(path)).unwrap();

    panic!("Database setup failed:\n{:#?}", error);
}

// --- Database Types ---

// --- User Type ---
pub struct User {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
    pub access_token: String,
    pub is_admin: bool,
}

impl User {
    /// Constructs a new `User`
    ///
    /// Note: The password field should be plaintext (it is hashed here)
    pub fn new(
        database: &Database,
        email: &String,
        first_name: &String,
        last_name: &String,
        password: &String,
        is_admin: bool,
    ) -> User {
        let hpassword = database.hash(password);

        let access_token = User::generate_access_token(database, email, &hpassword);

        User {
            email: email.clone(),
            first_name: first_name.clone(),
            last_name: last_name.clone(),
            password: hpassword,
            access_token,
            is_admin,
        }
    }

    fn generate_access_token(database: &Database, email: &String, hpassword: &String) -> String {
        let mut s = email.clone();
        s.push_str("::::");
        s.push_str(hpassword);

        database.hash(&s)
    }

    pub fn change_password(&mut self, database: &Database, hpassword: &String) {
        self.password = String::from(hpassword);
        self.access_token = User::generate_access_token(database, &self.email, &self.password);
    }
}
