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
                <li><a href="/tcde/3">/cde/3</a> - Get TCDE by ID</li>
                <li><a href="/event/1">/event/1</a> - Get Event by ID</li>
                <li><a href="/search_events?cde=[[850124],[734045]]&tcde=3">/search_events?cde=[[850124],[734045]]&tcde=3</a> - Search Events by CDE and TCDE</li>
                <li><a href="/event_detail/1073">/event_detail/1073</a> - Get Event Detail by ID</li>
                <li><a href="/eii_and?input1=[1]&input2=[112]">/eii_and?input1=[1]&input2=[112]</a> - Get subjects with both Event List 1 and Event List 2</li>
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
            apis::cde_api::search_cde,
            apis::cde_api::get_tcde,
            apis::event_api::get_event,
            apis::event_api::search_events,
            apis::event_api::get_event_detail,
            apis::eii_api::eii_and,
            ])

}