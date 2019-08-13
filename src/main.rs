extern crate crypto;
extern crate rusqlite;

mod database;
mod shared;
mod budget;
mod can_access_budget;
mod api;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};

use database::*;

use std::sync::{Mutex};

// Constants
const BINDING: &str = "127.0.0.1:3000";
const DB_PATH: &str = "budget.db";

// Shares database connection with all web server workers
struct AppState {
    database: Mutex<Database>
}

fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
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

    println!("Starting HTTP server using address \"{}\"...", BINDING);
    HttpServer::new(move || {
        App::new()
            .service(api::get_service())
            .register_data(state.clone())
            .route("/", web::get().to(index))
    })
    .bind(BINDING)
    .unwrap()
    .run()
    .unwrap();
}
