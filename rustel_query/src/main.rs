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
                .api-form {
                    margin-top: 10px;
                    padding: 10px;
                    background: #f8f9fa;
                    border-radius: 4px;
                    border: 1px solid #eee;
                }
                .form-group {
                    margin-bottom: 10px;
                }
                .form-group label {
                    display: block;
                    font-size: 0.9em;
                    margin-bottom: 3px;
                }
                .form-group input {
                    width: 100%;
                    padding: 6px 8px;
                    border: 1px solid #ddd;
                    border-radius: 3px;
                }
                .btn {
                    background: #0066cc;
                    color: white;
                    border: none;
                    padding: 8px 12px;
                    border-radius: 4px;
                    cursor: pointer;
                }
                .btn:hover {
                    background: #0055aa;
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
                            <div class="endpoint-description">Get MIMIC3 data element by ID</div>
                            <form class="api-form" onsubmit="callApiWithForm(event, '/cde/', this)">
                                <div class="form-group">
                                    <label for="cde-id">CDE ID:</label>
                                    <input type="text" id="cde-id" name="cde-id" value="2">
                                </div>
                                <button type="submit" class="btn">Send Request</button>
                                <span class="response-time"></span>
                            </form>
                        </li>
                        
                        <li class="endpoint-item">
                            <div class="endpoint-description">Search MIMIC3 data element by collection, field, value</div>
                            <form class="api-form" onsubmit="callApiWithForm(event, '/search_cde', this)">
                                <div class="form-group">
                                    <label for="collection">Collection:</label>
                                    <input type="text" id="collection" name="collection" value="d_icd_diagnoses">
                                </div>
                                <div class="form-group">
                                    <label for="field">Field:</label>
                                    <input type="text" id="field" name="field" value="long_title">
                                </div>
                                <div class="form-group">
                                    <label for="value">Value:</label>
                                    <input type="text" id="value" name="value" value="essential hypertension">
                                </div>
                                <div class="form-group">
                                    <label for="partial">Partial Match:</label>
                                    <select id="partial" name="partial">
                                        <option value="true" selected>true</option>
                                        <option value="false">false</option>
                                    </select>
                                </div>
                                <div class="form-group">
                                    <label for="limit">Limit:</label>
                                    <input type="number" id="limit" name="limit" value="10">
                                </div>
                                <button type="submit" class="btn">Send Request</button>
                                <span class="response-time"></span>
                            </form>
                        </li>
                        
                        <li class="endpoint-item">
                            <div class="endpoint-description">Get MIMIC3 temporal data element by ID</div>
                            <form class="api-form" onsubmit="callApiWithForm(event, '/tcde/', this)">
                                <div class="form-group">
                                    <label for="tcde-id">TCDE ID:</label>
                                    <input type="text" id="tcde-id" name="tcde-id" value="3">
                                </div>
                                <button type="submit" class="btn">Send Request</button>
                                <span class="response-time"></span>
                            </form>
                        </li>
                        
                        <li class="endpoint-item">
                            <div class="endpoint-description">Search Events by MIMIC3 data element and temporal data element</div>
                            <form class="api-form" onsubmit="callApiWithForm(event, '/search_events', this)">
                                <div class="form-group">
                                    <label for="cde">CDE (2D array format):</label>
                                    <input type="text" id="cde" name="cde" value="[[850124],[734045]]">
                                </div>
                                <div class="form-group">
                                    <label for="tcde">TCDE:</label>
                                    <input type="text" id="tcde" name="tcde" value="3">
                                </div>
                                <button type="submit" class="btn">Send Request</button>
                                <span class="response-time"></span>
                            </form>
                        </li>
                        
                        <li class="endpoint-item">
                            <div class="endpoint-description">TEL diamond matching query</div>
                            <form class="api-form" onsubmit="callApiWithForm(event, '/efcfcd_diamond', this)">
                                <div class="form-group">
                                    <label for="event_list1">Event List 1:</label>
                                    <input type="text" id="event_list1" name="event_list1" value="[1]">
                                </div>
                                <div class="form-group">
                                    <label for="event_list2">Event List 2:</label>
                                    <input type="text" id="event_list2" name="event_list2" value="[122]">
                                </div>
                                <div class="form-group">
                                    <label for="delta_max">Delta Max (milliseconds):</label>
                                    <input type="text" id="delta_max" name="delta_max" value="3153600000">
                                </div>
                                <div class="form-group">
                                    <label for="delta_max_op">Delta Max Operator:</label>
                                    <select id="delta_max_op" name="delta_max_op">
                                        <option value="lt" selected>less than</option>
                                        <option value="lte">less than or equal</option>
                                        <option value="gt">greater than</option>
                                        <option value="gte">greater than or equal</option>
                                    </select>
                                </div>
                                <div class="form-group">
                                    <label for="cooccurrence">Co-occurrence:</label>
                                    <select id="cooccurrence" name="cooccurrence">
                                        <option value="true" selected>true</option>
                                        <option value="false">false</option>
                                    </select>
                                </div>
                                <div class="form-group">
                                    <label for="negation">Negation:</label>
                                    <select id="negation" name="negation">
                                        <option value="false" selected>false</option>
                                        <option value="true">true</option>
                                    </select>
                                </div>
                                <button type="submit" class="btn">Send Request</button>
                                <span class="response-time"></span>
                            </form>
                        </li>
                        
                        <li class="endpoint-item">
                            <div class="endpoint-description">Search Events by OMOP concepts</div>
                            <form class="api-form" onsubmit="callApiWithForm(event, '/search_events_by_omop', this)">
                                <div class="form-group">
                                    <label for="omop_concepts">OMOP Concepts (JSON array):</label>
                                    <input type="text" id="omop_concepts" name="omop_concepts" value='["44826401","44825200"]'>
                                </div>
                                <button type="submit" class="btn">Send Request</button>
                                <span class="response-time"></span>
                            </form>
                        </li>
                    </ul>
                </div>
                
                <div class="result-panel">
                    <h2>Response</h2>
                    <div id="timing"></div>
                    <pre id="result">Fill out a form and click "Send Request" to see results here...</pre>
                </div>
            </div>

            <script>
            async function callApiWithForm(event, baseUrl, form) {
                event.preventDefault();
                const resultPanel = document.getElementById('result');
                const timingPanel = document.getElementById('timing');
                const responseTimeSpan = form.querySelector('.response-time');
                
                responseTimeSpan.innerHTML = '<div class="loading"></div>';
                resultPanel.textContent = 'Loading...';
                
                try {
                    // Build URL from form inputs
                    let url = baseUrl;
                    const formData = new FormData(form);
                    let params = new URLSearchParams();
                    let pathParam = '';
                    
                    // Process each form field
                    for (let [key, value] of formData.entries()) {
                        if (key.endsWith('-id')) {
                            // Handle path parameters (e.g., /cde/2)
                            pathParam = value;
                        } else {
                            // Handle query parameters
                            params.append(key, value);
                        }
                    }
                    
                    // Complete URL
                    if (pathParam) {
                        url += pathParam;
                    }
                    
                    const paramString = params.toString();
                    if (paramString) {
                        url += '?' + paramString;
                    }
                    
                    console.log('Calling API: ' + url);
                    
                    // Make API call
                    const start = performance.now();
                    const response = await fetch(url);
                    const end = performance.now();
                    
                    // Process timing
                    const serverTime = response.headers.get('X-Response-Time');
                    const clientTime = (end - start).toFixed(2);
                    const timing = `Server Processing: ${serverTime || 'N/A'}, Total Request: ${clientTime}ms`;
                    
                    responseTimeSpan.textContent = timing;
                    timingPanel.textContent = timing;
                    
                    // Process response
                    const data = await response.json();
                    resultPanel.textContent = JSON.stringify(data, null, 2);
                    
                    // Highlight active endpoint
                    document.querySelectorAll('.endpoint-item').forEach(item => 
                        item.style.backgroundColor = (item === form.closest('.endpoint-item')) ? '#f0f7ff' : '#fff'
                    );
                } catch (error) {
                    const errorMsg = 'Error: ' + error.message;
                    responseTimeSpan.textContent = errorMsg;
                    resultPanel.textContent = errorMsg;
                }
            }
            
            // For backwards compatibility with existing links
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