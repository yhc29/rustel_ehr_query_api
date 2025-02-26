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
            <p>TEL Query API is a RESTful API that provides access to the MIMIC3 database (46,520 subjects) using Temporal Ensemble Logic.</p>
            <h2>Endpoints</h2>
            <ul>
                <li><a href="/cde/2">/cde/2</a> - Get MIMIC3 data element by ID</li>

                <li><a href="/search_cde?collection=d_icd_diagnoses&field=long_title&value=essential hypertension&partial=true&limit=10">/search_cde?collection=d_icd_diagnoses&field=long_title&value=essential hypertension&partial=true&limit=10</a> - Search MIMIC3 data element by collection, field, value. Set partial to true for partial match. Set limit to limit the number of results</li>

                <li><a href="/tcde/3">/cde/3</a> - Get MIMIC3 temporal data element by ID</li>

                <li><a href="/event/1073">/event/1073</a> - Get Event by ID</li>

                <li><a href="/search_events?cde=[[850124],[734045]]&tcde=3">/search_events?cde=[[850124],[734045]]&tcde=3</a> - Search Events by MIMIC3 data element and temporal data element</li>

                <li><a href="/event_detail/1073">/event_detail/1073</a> - Get Event Detail by ID</li>

                <li><a href="/patient/10026">/patient/10026</a> - Get patient records by PTID</li>

                <li><a href="/patient_events/10026">/patient_events/10026</a> - Get patient Events by PTID</li>

                <li><a href="/eii_and?input1=[1]&input2=[112]">/eii_and?input1=[1]&input2=[112]</a> - Get subjects with at least one event from both input1 and input2</li>

                <li><a href="/efcfcd_diamond?event_list1=[1]&event_list2=[122]&delta_max=3153600000&delta_max_op=lt&cooccurrence=true&negation=false">/efcfcd_diamond?event_list1=[1]&event_list2=[122]&delta_max=3153600000&delta_max_op=lt&cooccurrence=true&negation=false</a> - TEL diamond matching query. Get subjects containing at least one event from event_list1 followed by an event from event_list2 within delta_max time</li>
                <li><a href="/efcfcd_diamond?event_list1=[1]&event_list2=[1073,11589,12312,12627,14852,14875,18854,30911,33224,45256,45702,5775,7267,11270,12311,12824,13502,14327,15974,17259,18748,19855,20078,20639,25043,25634,28529,31755,35050,35185,35518,38083,39229,41060,41251,42978,44781,48432,49728,54377,112,129,148,166,213,277,298,390,506,621,779,1456,1491,1648,2204,2338,2447,2765,2989,3097,3403,5626,6167,6906,7578,8467,9762,13575,18229,21782,24384,25395,40285,45310,45852,47310,11111,14049,21063,21224,21305,26658,40446,40448,44245,46374,52632,13476,14320,38313,42872]&delta_max=3153600000&delta_max_op=lt&cooccurrence=true&negation=false">/efcfcd_diamond?event_list1=[1]&event_list2=[1073,11589,12312,12627,14852,14875,18854,30911,33224,45256,45702,5775,7267,11270,12311,12824,13502,14327,15974,17259,18748,19855,20078,20639,25043,25634,28529,31755,35050,35185,35518,38083,39229,41060,41251,42978,44781,48432,49728,54377,112,129,148,166,213,277,298,390,506,621,779,1456,1491,1648,2204,2338,2447,2765,2989,3097,3403,5626,6167,6906,7578,8467,9762,13575,18229,21782,24384,25395,40285,45310,45852,47310,11111,14049,21063,21224,21305,26658,40446,40448,44245,46374,52632,13476,14320,38313,42872]&delta_max=3153600000&delta_max_op=lt&cooccurrence=true&negation=false</a> - Heavy query example for TEL diamond matching query.</li>
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
            apis::patient_api::get_patient,
            apis::patient_api::get_patient_events,
            apis::temporal_query_api::efcfcd_diamond_v4_1
            ])

}