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

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use database::*;

use std::sync::{Mutex};

// Constants
const BINDING: &str = "localhost:3000";
const DB_PATH: &str = "budget.db";

// Path to private key and certificate generated using:
//      openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem \
//          -days 365 -sha256 -subj "<subject here>"
const SSL_PRIVATE_KEY: &str = "";
const SSL_CERT: &str = "";

// Shares database connection with all web server workers
struct AppState {
    database: Mutex<Database>
}

fn main() {
    let secret = String::from("Ks&#j%1_7,~");

    println!("Budget Tracker server starting...");

    println!("Loading database...");
    let database = match Database::new(secret, DB_PATH) {
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
        .set_private_key_file(SSL_PRIVATE_KEY, SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file(SSL_CERT).unwrap();

    println!("Starting HTTPS server using address \"{}\"...", BINDING);
    HttpServer::new(move || {
        App::new()
            .service(api::get_service())
            .service(
                actix_files::Files::new("/", "public/.")
                    .index_file("index.html")
            )
            .register_data(state.clone())
    })
    .bind_ssl(BINDING, builder)
    .unwrap()
    .run()
    .unwrap();
}
