use std::env;
extern crate dotenv;

use dotenv::dotenv;

use mongodb::{
    bson::{doc, extjson::de::Error, oid::ObjectId, Document},
    results::{InsertOneResult, UpdateResult, DeleteResult},
    sync::{Client, Collection, Database},
};
use crate::models::cde::{CdeRecord, CDE};
use crate::models::tcde::TCDE;
use crate::models::event::{Event, EventRecord};
use crate::models::eii::Eii;
use crate::models::fc::FC;

pub struct MongoRepo {
    db: Database,
    pub cde_collection: Collection<CDE>,
    pub tcde_collection: Collection<TCDE>,
    pub event_collection: Collection<Event>,
    pub eii_collection: Collection<Eii>,
    pub fc_collection: Collection<FC>,
    pub cde_record_collection: Collection<CdeRecord>,
    pub event_record_collection: Collection<EventRecord>,
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
        let fc_collection = db.collection("fcs");
        let cde_record_collection = db.collection("cde_records");
        let event_record_collection = db.collection("event_records");

        MongoRepo { db, cde_collection, tcde_collection, event_collection, eii_collection, fc_collection, cde_record_collection,event_record_collection }
    }



}
