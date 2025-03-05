mod models;
mod database;
mod apis;
mod timing;

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
use rocket::config::{Config, TlsConfig};

use database::mongodb::{MongoRepo};
use mongodb::bson::{doc, Document,Bson};

#[get("/")]
fn index() -> RawHtml<String> {
    // the index page serves as user manual
    let html = r#"
    <html>
        <head>
            <title>TEL Query API</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    margin: 0;
                    padding: 20px;
                    line-height: 1.6;
                }
                .container {
                    display: flex;
                    gap: 20px;
                    max-width: 1600px;
                    margin: 0 auto;
                }
                .endpoints {
                    flex: 1;
                    min-width: 600px;
                }
                .result-panel {
                    flex: 1;
                    position: sticky;
                    top: 20px;
                    max-height: calc(100vh - 40px);
                    overflow-y: auto;
                    padding: 20px;
                    background: #f5f5f5;
                    border-radius: 5px;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                }
                .response-time {
                    color: #666;
                    font-size: 0.9em;
                    margin-left: 10px;
                }
                .endpoint-list {
                    list-style: none;
                    padding: 0;
                }
                .endpoint-item {
                    margin-bottom: 15px;
                    padding: 10px;
                    background: #fff;
                    border-radius: 4px;
                    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
                }
                .endpoint-link {
                    color: #0066cc;
                    text-decoration: none;
                    word-break: break-all;
                }
                .endpoint-link:hover {
                    text-decoration: underline;
                }
                .endpoint-description {
                    margin-top: 5px;
                    color: #666;
                }
                pre {
                    white-space: pre-wrap;
                    word-wrap: break-word;
                    background: #fff;
                    padding: 15px;
                    border-radius: 4px;
                    border: 1px solid #ddd;
                    overflow-x: auto;
                }
                .loading {
                    display: inline-block;
                    width: 20px;
                    height: 20px;
                    border: 3px solid #f3f3f3;
                    border-top: 3px solid #3498db;
                    border-radius: 50%;
                    animation: spin 1s linear infinite;
                }
                @keyframes spin {
                    0% { transform: rotate(0deg); }
                    100% { transform: rotate(360deg); }
                }
            </style>
        </head>
        <body>
            <h1>TEL Query API</h1>
            <p>TEL Query API is a RESTful API that provides access to the MIMIC3 database (46,520 subjects) using Temporal Ensemble Logic.</p>
            
            <div class="container">
                <div class="endpoints">
                    <h2>Endpoints</h2>
                    <ul class="endpoint-list">
                        <li class="endpoint-item">
                            <a href="/cde/2" class="endpoint-link" onclick="fetchWithTime(event, this)">/cde/2</a>
                            <span class="response-time"></span>
                            <div class="endpoint-description">Get MIMIC3 data element by ID</div>
                        </li>
                        <li class="endpoint-item">
                            <a href="/search_cde?collection=d_icd_diagnoses&field=long_title&value=essential hypertension&partial=true&limit=10" class="endpoint-link" onclick="fetchWithTime(event, this)">/search_cde?collection=d_icd_diagnoses&field=long_title&value=essential hypertension&partial=true&limit=10</a>
                            <span class="response-time"></span>
                            <div class="endpoint-description">Search MIMIC3 data element by collection, field, value. Set partial to true for partial match. Set limit to limit the number of results</div>
                        </li>
                        <li class="endpoint-item">
                            <a href="/tcde/3" class="endpoint-link" onclick="fetchWithTime(event, this)">/tcde/3</a>
                            <span class="response-time"></span>
                            <div class="endpoint-description">Get MIMIC3 temporal data element by ID</div>
                        </li>
                        <li class="endpoint-item">
                            <a href="/event/1073" class="endpoint-link" onclick="fetchWithTime(event, this)">/event/1073</a>
                            <span class="response-time"></span>
                            <div class="endpoint-description">Get Event by ID</div>
                        </li>
                        <li class="endpoint-item">
                            <a href="/search_events?cde=[[850124],[734045]]&tcde=3" class="endpoint-link" onclick="fetchWithTime(event, this)">/search_events?cde=[[850124],[734045]]&tcde=3</a>
                            <span class="response-time"></span>
                            <div class="endpoint-description">Search Events by MIMIC3 data element and temporal data element</div>
                        </li>
                        <li class="endpoint-item">
                            <a href="/event_detail/1073" class="endpoint-link" onclick="fetchWithTime(event, this)">/event_detail/1073</a>
                            <span class="response-time"></span>
                            <div class="endpoint-description">Get Event Detail by ID</div>
                        </li>
                        <li class="endpoint-item">
                            <a href="/patient/10026" class="endpoint-link" onclick="fetchWithTime(event, this)">/patient/10026</a>
                            <span class="response-time"></span>
                            <div class="endpoint-description">Get patient records by PTID</div>
                        </li>
                        <li class="endpoint-item">
                            <a href="/patient_events/10026" class="endpoint-link" onclick="fetchWithTime(event, this)">/patient_events/10026</a>
                            <span class="response-time"></span>
                            <div class="endpoint-description">Get patient Events by PTID</div>
                        </li>
                        <li class="endpoint-item">
                            <a href="/eii_and?input1=[1]&input2=[112]" class="endpoint-link" onclick="fetchWithTime(event, this)">/eii_and?input1=[1]&input2=[112]</a>
                            <span class="response-time"></span>
                            <div class="endpoint-description">Get subjects with at least one event from both input1 and input2</div>
                        </li>
                        <li class="endpoint-item">
                            <a href="/efcfcd_diamond?event_list1=[1]&event_list2=[122]&delta_max=3153600000&delta_max_op=lt&cooccurrence=true&negation=false" class="endpoint-link" onclick="fetchWithTime(event, this)">/efcfcd_diamond?event_list1=[1]&event_list2=[122]&delta_max=3153600000&delta_max_op=lt&cooccurrence=true&negation=false</a>
                            <span class="response-time"></span>
                            <div class="endpoint-description">TEL diamond matching query. Get subjects containing at least one event from event_list1 followed by an event from event_list2 within delta_max time</div>
                        </li>
                        <li class="endpoint-item">
                            <a href="/efcfcd_diamond?event_list1=[1]&event_list2=[1073,11589,12312,12627,14852,14875,18854,30911,33224,45256,45702,5775,7267,11270,12311,12824,13502,14327,15974,17259,18748,19855,20078,20639,25043,25634,28529,31755,35050,35185,35518,38083,39229,41060,41251,42978,44781,48432,49728,54377,112,129,148,166,213,277,298,390,506,621,779,1456,1491,1648,2204,2338,2447,2765,2989,3097,3403,5626,6167,6906,7578,8467,9762,13575,18229,21782,24384,25395,40285,45310,45852,47310,11111,14049,21063,21224,21305,26658,40446,40448,44245,46374,52632,13476,14320,38313,42872]&delta_max=3153600000&delta_max_op=lt&cooccurrence=true&negation=false" class="endpoint-link" onclick="fetchWithTime(event, this)">/efcfcd_diamond?event_list1=[1]&event_list2=[1073,11589,12312,12627,14852,14875,18854,30911,33224,45256,45702,5775,7267,11270,12311,12824,13502,14327,15974,17259,18748,19855,20078,20639,25043,25634,28529,31755,35050,35185,35518,38083,39229,41060,41251,42978,44781,48432,49728,54377,112,129,148,166,213,277,298,390,506,621,779,1456,1491,1648,2204,2338,2447,2765,2989,3097,3403,5626,6167,6906,7578,8467,9762,13575,18229,21782,24384,25395,40285,45310,45852,47310,11111,14049,21063,21224,21305,26658,40446,40448,44245,46374,52632,13476,14320,38313,42872]&delta_max=3153600000&delta_max_op=lt&cooccurrence=true&negation=false</a>
                            <span class="response-time"></span>
                            <div class="endpoint-description">Heavy query example for TEL diamond matching query.</div>
                        </li>
                        <li class="endpoint-item">
                            <a href='/search_events_by_omop?omop_concepts=["44826401","44825200"]' class="endpoint-link" onclick="fetchWithTime(event, this)">/search_events_by_omop?omop_concepts=["44826401","44825200"]</a>
                            <span class="response-time"></span>
                            <div class="endpoint-description">Search Events by OMOP concepts</div>
                        </li>

                    </ul>
                </div>
                
                <div class="result-panel">
                    <h2>Response</h2>
                    <div id="timing"></div>
                    <pre id="result">Click an API endpoint to see results here...</pre>
                </div>
            </div>

            <script>
            async function fetchWithTime(event, link) {
                event.preventDefault();
                const timeSpan = link.nextElementSibling;
                const resultPanel = document.getElementById('result');
                const timingPanel = document.getElementById('timing');
                
                timeSpan.innerHTML = '<div class="loading"></div>';
                resultPanel.textContent = 'Loading...';
                
                try {
                    const start = performance.now();
                    const response = await fetch(link.href);
                    const end = performance.now();
                    
                    const serverTime = response.headers.get('X-Response-Time');
                    const clientTime = (end - start).toFixed(2);
                    const timing = `Server Processing: ${serverTime || 'N/A'}, Total Request: ${clientTime}ms`;
                    
                    timeSpan.textContent = timing;
                    timingPanel.textContent = timing;
                    
                    const data = await response.json();
                    resultPanel.textContent = JSON.stringify(data, null, 2);
                    
                    // Highlight the active endpoint
                    document.querySelectorAll('.endpoint-item').forEach(item => 
                        item.style.backgroundColor = item.contains(link) ? '#f0f7ff' : '#fff'
                    );
                } catch (error) {
                    const errorMsg = 'Error: ' + error.message;
                    timeSpan.textContent = errorMsg;
                    resultPanel.textContent = errorMsg;
                }
            }
            </script>
        </body>
    </html>
    "#;
    RawHtml(html.to_string())

}

#[launch]
fn rocket() -> _ {
    let config = Config::figment()
        .merge(("tls", TlsConfig::from_paths(
            "certs/cert.pem",
            "certs/private.key"
        )));

    let tel_db = MongoRepo::init();
    rocket::custom(config)
        .attach(timing::RequestTimer)
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
            apis::temporal_query_api::efcfcd_diamond_v4_1,
            apis::event_api::search_events_by_omop
            ])

}