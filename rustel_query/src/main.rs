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

use database::mongodb::{MongoRepo};
use mongodb::bson::{doc, Document,Bson};

#[get("/")]
fn index() -> RawHtml<String> {
    // the index page serves as user manual
    let html = r#"
    <html>
        <head>
            <title>TEL Query API</title>
        </head>
        <body>
            <h1>TEL Query API</h1>
            <p>TEL Query API is a RESTful API that provides access to the TEL database.</p>
            <h2>Endpoints</h2>
            <ul>
                <li><a href="/cde/1">/cde/1</a> - Get CDE by ID</li>
                <li><a href="/search_cde?collection=d_icd_diagnoses&field=long_title&value=essential hypertension&partial=true&limit=10">/search_cde?collection=d_icd_diagnoses&field=long_title&value=essential hypertension&partial=true&limit=10</a> - Search CDE by collection, field, value, partial, and limit</li>
            </ul>
        </body>
    </html>
    "#;
    RawHtml(html.to_string())
}

#[launch]
fn rocket() -> _ {
    let tel_db = MongoRepo::init();
    rocket::build()
        .manage(tel_db)
        .mount("/", routes![
            index, 
            apis::cde_api::get_cde, 
            apis::cde_api::search_cde])

}