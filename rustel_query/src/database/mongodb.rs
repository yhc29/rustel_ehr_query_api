use std::env;
extern crate dotenv;

use dotenv::dotenv;

use mongodb::{
    bson::{doc, extjson::de::Error, oid::ObjectId, Document},
    results::{InsertOneResult, UpdateResult, DeleteResult},
    sync::{Client, Collection, Database},
};
use crate::models::cde::CDE;
use crate::models::tcde::TCDE;
use crate::models::event::Event;
use crate::models::eii::Eii;

pub struct MongoRepo {
    db: Database,
    pub cde_collection: Collection<CDE>,
    pub tcde_collection: Collection<TCDE>,
    pub event_collection: Collection<Event>,
    pub eii_collection: Collection<Eii>,
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
        let eii_collection = db.collection("eii");

        MongoRepo { db, cde_collection, tcde_collection, event_collection, eii_collection }
    }



}
