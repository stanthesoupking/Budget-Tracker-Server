extern crate crypto;
extern crate rusqlite;
extern crate chrono;

mod database;
mod shared;
mod budget;
mod transaction;
mod can_access_budget;
mod budget_period;
mod api;
mod util;
mod config;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use database::*;
use config::Config;

use std::sync::{Mutex};

// Constants
const DB_PATH: &str = "budget.db";

// Shares database connection with all web server workers
struct AppState {
    database: Mutex<Database>
}

fn main() {
    println!("Budget Tracker server starting...");

    // Load config
    println!("Loading config...");
    let config = Config::load();

    println!("Loading database...");
    let database = match Database::new(config.secret.clone(), DB_PATH) {
        Ok(database) => database,
        Err(err) => {
            panic!("Error occurred while loading database: {:?}", err);
        }
    };

    let state = web::Data::new(AppState {
        database: Mutex::new(database)
    });

    println!("Loading SSL keys...");
    let mut builder =
        SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();

    builder
        .set_private_key_file(&config.ssl_key_path, SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file(&config.ssl_cert_path).unwrap();

    println!("Starting HTTPS server using address \"{}\"...", &config.binding);
    HttpServer::new(move || {
        App::new()
            .service(api::get_service())
            .service(
                actix_files::Files::new("/", "public/.")
                    .index_file("index.html")
            )
            .register_data(state.clone())
    })
    .bind_ssl(&config.binding, builder)
    .unwrap()
    .run()
    .unwrap();
}
