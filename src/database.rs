
use crate::budget::Budget;

use std::fs;
use std::path::Path;
use std::io;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

use rusqlite::{Connection, params, NO_PARAMS};
use rusqlite::Error::SqliteFailure;

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
    UnknownError
}

impl std::convert::From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Error::UnknownError
    }
}

pub struct Database {
    db_conn: Connection,
    secret: String 
}

impl Database {
    pub fn new(secret: String, path: &str) -> Result<Database, Error> {
        let rpath = Path::new(path);

        // Check if db file exist
        let init_req = !rpath.exists();

        // Connect to sqlite database
        let db_conn = match Connection::open(path) {
            Ok(conn) => conn,
            Err(_) => return Err(Error::LoadFileError)
        };

        // Enable foreign key support
        db_conn.execute("PRAGMA foreign_keys = ON", NO_PARAMS)
            .expect("Failed enabling foreign key support.");

        let database = Database {
            secret,
            db_conn
        };

        // Does the database need to be initialised?
        if init_req {

            match database.init() {
                Err(error) => rollback(path, error),
                Ok(_) => {
                    // Create admin user
                    println!(" === Admin User Setup ===");

                    println!("Username:");

                    let mut buffer = String::new();

                    io::stdin().read_line(&mut buffer)
                        .expect("Failed reading line.");
                    
                    let username = buffer;

                    let password: String;
                    loop {
                        println!("Password:");
                        let buffer = io::stdin().read_passwd(&mut io::stdout());

                        match buffer {
                            Ok(Some(p)) => { 
                                password = p;
                                break;
                            },
                            _ => ()
                        }
                    }

                    io::stdin().read_passwd(&mut io::stdout())
                        .expect("Failed reading password");

                    let admin_user = User::new(&database, &username, &password, true);

                    match database.insert_user(&admin_user) {
                        Ok(_) => {
                            println!("Admin user created.");
                        },
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
                username TEXT NOT NULL PRIMARY KEY,
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
                FOREIGN KEY(owner) REFERENCES users(username)
            );

            CREATE TABLE can_access_budget (
                budget_id INTEGER NOT NULL,
                username TEXT NOT NULL,
                PRIMARY KEY(budget_id, username),
                FOREIGN KEY(budget_id) REFERENCES budgets(budget_id),
                FOREIGN KEY(username) REFERENCES users(username)
            );

            CREATE TABLE recurring_transactions (
                recurring_transaction_id INTEGER PRIMARY KEY AUTOINCREMENT,
                budget_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                note TEXT NOT NULL,
                amount FLOAT NOT NULL,
                day_in_period INTEGER NOT NULL,
                FOREIGN KEY(budget_id) REFERENCES budgets(budget_id)
            );

            CREATE TABLE transactions (
                transaction_id INTEGER PRIMARY KEY AUTOINCREMENT,
                budget_id INTEGER NOT NULL,
                username INTEGER NOT NULL,
                name TEXT NOT NULL,
                note TEXT NOT NULL,
                date DATE NOT NULL,
                amount FLOAT NOT NULL,
                FOREIGN KEY(budget_id) REFERENCES budgets(budget_id)
                FOREIGN KEY(username) REFERENCES users(username)
            );
            "
        ) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::UnknownError)
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
                username, password, access_token, is_admin
            )
            VALUES(?1, ?2, ?3, ?4)",
            params![user.username, user.password, user.access_token, user.is_admin]);

        match res {
            Ok(_) => Ok(()),
            Err(error) => match error {
                SqliteFailure(error, _) => {
                    match error.code {
                        rusqlite::ErrorCode::ConstraintViolation => Err(Error::UserAlreadyExists),
                        _ => Err(Error::UnknownError)
                    }
                },
                _ => Err(Error::UnknownError)
            }
        }
    }

    pub fn get_user_by_username(&self, username: &str) -> Result<Option<User>, Error> {
        let mut stmt = self.db_conn.prepare(
            "SELECT username, password, access_token, is_admin
            FROM users WHERE username = ?1"
        )?;

        match stmt.query_row(params![username], |row| {
            Ok(User {
                username: row.get(0)?,
                password: row.get(1)?,
                access_token: row.get(2)?,
                is_admin: row.get(3)?,
            })
        }) {
            Ok(user) => Ok(Some(user)),
            Err(error) => match error {
                QueryReturnedNoRows => Ok(None),
                _ => Err(Error::UnknownError)
            }
        }
    }

    pub fn get_user_by_access_token(&self, access_token: &str) -> Result<Option<User>, Error> {
        let mut stmt = self.db_conn.prepare(
            "SELECT username, password, access_token, is_admin
            FROM users WHERE access_token = ?1"
        )?;

        match stmt.query_row(params![access_token], |row| {
            Ok(User {
                username: row.get(0)?,
                password: row.get(1)?,
                access_token: row.get(2)?,
                is_admin: row.get(3)?,
            })
        }) {
            Ok(user) => Ok(Some(user)),
            Err(error) => match error {
                QueryReturnedNoRows => Ok(None),
                _ => Err(Error::UnknownError)
            }
        }
    }

    pub fn update_user(&self, user: &User) -> Result<(), Error> {

        let res = self.db_conn.execute(
            "UPDATE users SET password = ?1, access_token = ?2, is_admin = ?3
            WHERE username = ?4",
            params![
                user.password,
                user.access_token,
                user.is_admin,
                user.username
            ]
        );

        match res {
            Ok(_) => Ok(()),
            Err(error) => match error {
                SqliteFailure(error, desc) => Err(Error::SqliteError(error, desc)),
                _ => Err(Error::UnknownError)
            }
        }
    }

    pub fn get_available_budgets(&self, access_token: &str) -> Result<Vec<Budget>, Error> {
        let mut stmt = self.db_conn.prepare(
            "SELECT budget_id, owner, name, spend_limit, period_length FROM budgets WHERE budget_id in (
            SELECT budget_id FROM (SELECT budget_id, owner AS username FROM budgets
            UNION SELECT budget_id, username FROM can_access_budget) WHERE username in
            (SELECT username FROM users WHERE access_token = ?1))"
        )?;

        let mut result: Vec<Budget> = Vec::new();

        let budget_iter = stmt.query_map(params![access_token], |row| {
            Ok(Budget {
                budget_id: row.get(0)?,
                owner: row.get(1)?,
                name: row.get(2)?,
                spend_limit: row.get(3)?,
                period_length: row.get(4)?,
            })
        });

        for budget in budget_iter? {
            result.push(budget?);
        }

        Ok(result)
    }

    pub fn add_budget(&self, access_token: &str, budget: &Budget) -> Result<Budget, Error> {
        let user = match self.get_user_by_access_token(access_token) {
            Ok(user) => match user {
                Some(user) => user,
                None => return Err(Error::InvalidCredentials)
            },
            Err(error) => return Err(error)
        };

        let res = self.db_conn.execute(
            "INSERT INTO budgets(
                owner, name, spend_limit, period_length
            )
            VALUES(?1, ?2, ?3, ?4)",
            params![user.username, budget.name, budget.spend_limit, budget.period_length]);

        match res {
            Ok(_) => {
                let budget_id = self.db_conn.last_insert_rowid();
                Ok(Budget {
                    budget_id: Some(budget_id),
                    owner: Some(user.username),
                    name: budget.name.clone(),
                    spend_limit: budget.spend_limit,
                    period_length: budget.period_length
                })
            },
            Err(error) => match error {
                SqliteFailure(error, desc) => Err(Error::SqliteError(error, desc)),
                _ => Err(Error::UnknownError)
            }
        }
    }

    pub fn get_budget(&self, budget_id: i64) -> Result<Option<Budget>, Error> {
        // Get budget
        let mut stmt = self.db_conn.prepare(
            "SELECT budget_id, owner, name, spend_limit, period_length FROM budgets
            WHERE budget_id = ?1"
        )?;

        match stmt.query_row(params![budget_id], |row| {
            Ok(Budget {
                budget_id: row.get(0)?,
                owner: row.get(1)?,
                name: row.get(2)?,
                spend_limit: row.get(3)?,
                period_length: row.get(4)?,
            })
        }) {
            Ok(budget) => Ok(Some(budget)),
            Err(error) => match error {
                QueryReturnedNoRows => Ok(None),
                _ => Err(Error::UnknownError)
            }
        }
    }

    pub fn get_available_budget(&self, access_token: &str, budget_id: i64) -> Result<Option<Budget>, Error> {
        // Get available budget
        let mut stmt = self.db_conn.prepare(
            "SELECT budget_id, owner, name, spend_limit, period_length FROM budgets WHERE budget_id = ?1 AND budget_id in (
            SELECT budget_id FROM (SELECT budget_id, owner AS username FROM budgets
            UNION SELECT budget_id, username FROM can_access_budget) WHERE username in
            (SELECT username FROM users WHERE access_token = ?2))"
        )?;

        match stmt.query_row(params![budget_id, access_token], |row| {
            Ok(Budget {
                budget_id: row.get(0)?,
                owner: row.get(1)?,
                name: row.get(2)?,
                spend_limit: row.get(3)?,
                period_length: row.get(4)?,
            })
        }) {
            Ok(budget) => Ok(Some(budget)),
            Err(error) => match error {
                QueryReturnedNoRows => Ok(None),
                _ => Err(Error::UnknownError)
            }
        }
    }

    pub fn delete_budget(&self, access_token: &str, budget_id: i64) -> Result<(), Error> {
        let user = match self.get_user_by_access_token(access_token) {
            Ok(user) => match user {
                Some(user) => user,
                None => return Err(Error::InvalidCredentials)
            },
            Err(error) => return Err(error)
        };

        // Get budget
        let budget = self.get_budget(budget_id)?;

        match budget {
            Some(budget) => {
                let owner = match budget.owner {
                    Some(id) => id,
                    None => return Err(Error::UnknownError)
                };

                // Check if user is the budget owner
                if owner != user.username {
                    return Err(Error::UserDeniedError);
                }

                // Perform deletion
                let res = self.db_conn.execute(
                    "DELETE FROM budgets WHERE budget_id = ?1",
                    params![budget_id]);
                
                match res {
                    Ok(_) => Ok(()),
                    Err(error) => match error {
                        SqliteFailure(error, desc) => Err(Error::SqliteError(error, desc)),
                        _ => Err(Error::UnknownError)
                    }
                }
            },
            None => Err(Error::EntryNotFound)
        }
    }

    pub fn get_available_can_access_budget_users(&self, access_token: &str, budget_id: i64) -> Result<Vec<String>, Error> {
        let user = match self.get_user_by_access_token(access_token) {
            Ok(user) => match user {
                Some(user) => user,
                None => return Err(Error::InvalidCredentials)
            },
            Err(error) => return Err(error)
        };

        let mut stmt = self.db_conn.prepare(
            "SELECT username FROM users WHERE username IN (SELECT username FROM (SELECT owner AS username FROM budgets WHERE budget_id = ?1
            UNION SELECT username FROM can_access_budget WHERE budget_id = ?1))"
        )?;

        let mut is_available = false;
        let mut result: Vec<String> = Vec::new();

        let username_iter = stmt.query_map(params![budget_id], |row| {
            Ok(row.get(0)?)
        });

        for username in username_iter? {
            let username = username?;

            if username == user.username {
                is_available = true;
            }

            result.push(username);
        }

        if result.len() == 0 {
            Err(Error::EntryNotFound)
        } else if is_available {
            Ok(result)
        } else {
            Err(Error::UserDeniedError)
        }
    }

    pub fn add_can_access_budget(&self, access_token: &str, budget_id: i64,
        username: &str) -> Result<(), Error> {

        // Get current user
        let user = match self.get_user_by_access_token(access_token) {
            Ok(user) => match user {
                Some(user) => user,
                None => return Err(Error::InvalidCredentials)
            },
            Err(error) => return Err(error)
        };

        // Get budget
        let budget = match self.get_budget(budget_id) {
            Ok(budget) => match budget {
                Some(budget) => budget,
                None => return Err(Error::EntryNotFound)
            },
            Err(error) => return Err(error)
        };

        // Check if the current user is the budget owner
        match budget.owner {
            Some(owner) => {
                if !owner.eq(&user.username) {
                    return Err(Error::UserDeniedError);
                }
            },
            None => return Err(Error::UnknownError)
        };

        // Check if the request is trying to give owner access to their own budget
        if username.eq(&user.username) {
            return Err(Error::AccessRecursionError);
        }

        let res = self.db_conn.execute(
            "INSERT INTO can_access_budget(
                budget_id, username
            )
            VALUES(?1, ?2)",
            params![budget_id, username]);

        match res {
            Ok(_) => Ok(()),
            Err(error) => match error {
                SqliteFailure(error, desc) => Err(Error::SqliteError(error, desc)),
                _ => Err(Error::UnknownError)
            }
        }
    }

    pub fn delete_can_access_budget(&self, access_token: &str, budget_id: i64, username: &str) -> Result<(), Error> {
        let user = match self.get_user_by_access_token(access_token) {
            Ok(user) => match user {
                Some(user) => user,
                None => return Err(Error::InvalidCredentials)
            },
            Err(error) => return Err(error)
        };

        // Get budget
        let budget = self.get_budget(budget_id)?;

        match budget {
            Some(budget) => {
                let owner = match budget.owner {
                    Some(id) => id,
                    None => return Err(Error::UnknownError)
                };

                // Check if user is the budget owner
                if owner != user.username {
                    return Err(Error::UserDeniedError);
                }

                // Perform deletion
                let res = self.db_conn.execute(
                    "DELETE FROM can_access_budget WHERE budget_id = ?1 AND username = ?2",
                    params![budget_id, username]);
                
                match res {
                    Ok(_) => Ok(()),
                    Err(error) => match error {
                        SqliteFailure(error, desc) => Err(Error::SqliteError(error, desc)),
                        _ => Err(Error::UnknownError)
                    }
                }
            },
            None => Err(Error::EntryNotFound)
        }
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
    pub username: String,
    pub password: String,
    pub access_token: String,
    pub is_admin: bool
}

impl User {
    
    /// Constructs a new `User`
    /// 
    /// Note: The password field should be plaintext (it is hashed here)
    pub fn new(database: &Database, username: &String, password: &String, is_admin: bool) -> User {
        let hpassword = database.hash(password);

        let access_token = User::generate_access_token(database, username, &hpassword);

        User {
            username: username.clone(),
            password: hpassword,
            access_token,
            is_admin
        }
    }

    fn generate_access_token(database: &Database, username: &String, hpassword: &String) -> String {
        let mut s = username.clone();
        s.push_str("::::");
        s.push_str(hpassword);

        database.hash(&s)
    }

    pub fn change_password(&mut self, database: &Database, hpassword: &String) {
        self.password = String::from(hpassword);
        self.access_token = User::generate_access_token(database, &self.username, &self.password);
    }
}