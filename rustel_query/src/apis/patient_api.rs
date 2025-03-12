use crate::models::event::{EventRecordDetail,Event,EventRecord};
use crate::models::cde::{CdeRecord, CDE};
use crate::models::tcde;
use crate::{models::tcde::TCDE, database::mongodb::MongoRepo};
use mongodb::results::{self, InsertOneResult};
use rocket::{http::Status, serde::json::Json, State};
use mongodb::{options::ClientOptions, Client, bson::{self, doc}, options::FindOptions};
use mongodb::bson::Regex;
use serde::Serialize;
use rocket::request::FromParam;
use serde_json::Value;
use percent_encoding::percent_decode_str;

use crate::apis::event_api::{search_events_by_omop,StringArrayParam};
use super::{cde_api, event_api};

// create a structure of patient timeline record with a time field and event field
#[derive(Debug, Serialize)]
pub struct PatientTimelineRecord {
    time: String,
    event: Value,
}

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

#[get("/patient_timeline?<ptid>&<event_id_list>")]
pub fn get_patient_timeline(db: &State<MongoRepo>, ptid: &str, event_id_list: Option<StringArrayParam>) -> Result<Json<Vec<PatientTimelineRecord>>, Status> {
  // check if event_id_list is provided
  let filter = if event_id_list.is_none() {
    doc! {"ptid": ptid}
  } else {
    // map each id in event_id_list to integer
    let event_id_list = event_id_list.unwrap().0.iter().map(|id| id.parse::<i32>().unwrap()).collect::<Vec<i32>>();

    doc! {"ptid": ptid, "event_id": {"$in": event_id_list}}
  };
  let cursor = db
    .event_record_collection
    .find(filter, None)
    .map_err(|_| Status::InternalServerError)?;

  let mut results = Vec::new();

  let mut event_summary_dict: std::collections::HashMap<i32, Value> = std::collections::HashMap::new();

  for result in cursor {
      let doc = result.map_err(|_| Status::InternalServerError)?;
      let time = doc.time;
      let event_id = doc.event_id;
      let event_summary = if !event_summary_dict.contains_key(&event_id) {
        let summary = event_api::get_event_summary(db, &event_id.to_string()).map_err(|_| Status::InternalServerError)?.into_inner();
        event_summary_dict.insert(event_id, summary.clone());
        summary
      } else {
        event_summary_dict.get(&event_id).unwrap().clone()
      };
      results.push(PatientTimelineRecord {
          time: time.to_string(),
          event: event_summary,
      });
  }
  // sort by time
  results.sort_by(|a, b| a.time.cmp(&b.time));

  Ok(Json(results))
}