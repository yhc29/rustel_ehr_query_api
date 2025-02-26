use crate::models::event::{EventRecordDetail,Event,EventRecord};
use crate::models::cde::{CdeRecord, CDE};
use crate::models::tcde;
use crate::{models::tcde::TCDE, database::mongodb::MongoRepo};
use mongodb::results::{self, InsertOneResult};
use rocket::{http::Status, serde::json::Json, State};
use mongodb::{options::ClientOptions, Client, bson::doc, options::FindOptions};
use mongodb::bson::Regex;
use serde::Serialize;
use rocket::request::FromParam;
use serde_json::Value;
use percent_encoding::percent_decode_str;

use super::{cde_api, event_api};

#[get("/patient/<ptid>")]
pub fn get_patient(db: &State<MongoRepo>, ptid: &str) -> Result<Json<Vec<CdeRecord>>, Status> {
    let filter = doc! {"ptid": ptid.to_string()};
    let cursor= db
      .cde_record_collection
      .find(filter, None)
      .map_err(|_| Status::InternalServerError)?;

    let mut results = Vec::new();

    for result in cursor {
      let doc = result.map_err(|_| Status::InternalServerError)?;
      // let cde_list: Vec<CDE> = doc.cde.iter().map(|id| {
      //     let filter = doc! {"id": id};
      //     let cde_detail: Option<CDE> = db
      //         .cde_collection
      //         .find_one(filter, None)
      //         .ok()
      //         .expect("Error getting cde's detail");
      //     cde_detail.ok_or(Status::InternalServerError)
      // }).collect::<Result<Vec<CDE>, Status>>()?;
      results.push(doc);
    }
    Ok(Json(results))
}

#[get("/patient_events/<ptid>")]
pub fn get_patient_events(db: &State<MongoRepo>, ptid: &str) -> Result<Json<Vec<EventRecord>>, Status> {
    let filter = doc! {"ptid": ptid};
    let cursor = db
      .event_record_collection
      .find(filter, None)
      .map_err(|_| Status::InternalServerError)?;

    let mut results = Vec::new();

    for result in cursor {
        let doc = result.map_err(|_| Status::InternalServerError)?;
        // let time = doc.time;
        // let event_id = doc.event_id;
        // let event_detail = event_api::get_event_detail(db, &event_id.to_string()).map_err(|_| Status::InternalServerError)?;
        // let cde_list = event_detail.cde.clone();
        // let tcde = event_detail.tcde.clone();
        // results.push(EventRecordDetail {
        //     time,
        //     cde: cde_list,
        //     tcde,
        // });
        results.push(doc);
    }
    Ok(Json(results))
}