mod models;
mod database;
mod apis;

#[macro_use] extern crate rocket;
use std::env;
use rocket::{get, http::Status, serde::json::Json, State};
use rocket::http::ContentType;
use rocket::response::Responder;
use rocket::Request;
use rocket::response::Response;
use std::io::Cursor;
use std::time::Instant;
use rocket::response::content::RawHtml;
use rocket::form::Form;
use rocket::local::blocking::Client;

use apis::cde_api::{get_cde};
use database::mongodb::{MongoRepo};
use mongodb::bson::{doc, Document,Bson};

#[launch]
fn rocket() -> _ {
    let tel_db = MongoRepo::init();
    rocket::build()
        .manage(tel_db)
        .mount("/", routes![get_cde])

}