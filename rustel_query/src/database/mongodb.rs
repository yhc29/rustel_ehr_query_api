use std::env;
extern crate dotenv;

use dotenv::dotenv;

use mongodb::{
    bson::{doc, extjson::de::Error, oid::ObjectId, Document},
    results::{InsertOneResult, UpdateResult, DeleteResult},
    sync::{Client, Collection, Database},
};
use crate::models::event::Event;

pub struct MongoRepo {
    db: Database,
    pub cde_collection: Collection<CDE>,
    pub tcde_collection: Collection<TCDE>,
    pub event_collection: Collection<Event>,
}

impl MongoRepo {
    pub fn init() -> Self {
        dotenv().ok();
        let uri = match env::var("MONGO_URI") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error loading env variable"),
        };
        let client = Client::with_uri_str(uri).unwrap();
        let tel_db_name = match env::var("TEL_DB_NAME") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error loading env variable"),
        };
        let db = client.database(&tel_db_name);
        let cde_collection = db.collection("cde");
        let tcde_collection = db.collection("temporal_cde");
        let event_collection = db.collection("events");

        MongoRepo { db, cde_collection, tcde_collection, event_collection }
    }


}
