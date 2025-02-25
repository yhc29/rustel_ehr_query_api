use crate::{models::event::Event, models::cde::CDE, models::tcde::TCDE, database::mongodb::MongoRepo};
use mongodb::change_stream::event;
use mongodb::results::{self, InsertOneResult};
use rocket::{http::Status, serde::json::Json, State};
use mongodb::{options::ClientOptions, Client, bson::doc, options::FindOptions};
use mongodb::bson::Regex;
use serde::Serialize;
use rocket::request::FromParam;
use serde_json::Value;
use percent_encoding::percent_decode_str;

use std::collections::HashMap;

#[derive(Debug)]
pub struct EventListParam(Vec<i32>);

#[rocket::async_trait]
impl<'r> rocket::form::FromFormField<'r> for EventListParam {
    fn from_value(field: rocket::form::ValueField<'r>) -> rocket::form::Result<'r, Self> {
        // Decode URL parameter
        let decoded = percent_decode_str(field.value)
            .decode_utf8()
            .map_err(|_| rocket::form::Error::validation("Failed to decode URL"))?
            .to_string();
        
        // Remove brackets and split by commas
        let numbers = decoded
            .trim_matches(|c| c == '[' || c == ']')
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<i32>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| rocket::form::Error::validation("Invalid number in array"))?;
            
        Ok(EventListParam(numbers))
    }
}

#[derive(Debug, Serialize)]
pub struct CandidateResponse {
    #[serde(flatten)]
    groups: HashMap<i32, Vec<String>>
}

#[get("/eii_and?<input1>&<input2>")]
pub async fn eii_and(
  db: &State<MongoRepo>,
  input1: EventListParam,
  input2: EventListParam
) -> Result<Json<CandidateResponse>, Status> {
  let event_list1: Vec<i32> = input1.0;
  let event_list2: Vec<i32> = input2.0;
  // Combine both event lists for the $in operator
  let combined_events = event_list1.iter().chain(&event_list2).cloned().collect::<Vec<_>>();
  println!("combined_events: {:?}", combined_events);
  // Build the aggregation pipeline
  let pipeline = vec![
      doc! {
          "$match": {
              "pt_group": {"$gte": 0},
              "event": {"$in": combined_events}
          }
      },
      doc! {
          "$unwind": "$ptids"
      },
      doc! {
          "$group": {
              "_id": "$ptids",
              "pt_group": {"$first": "$pt_group"},
              "e1": {
                  "$sum": {
                      "$cond": [
                          {"$in": ["$event", event_list1]},
                          1,
                          0
                      ]
                  }
              },
              "e2": {
                  "$sum": {
                      "$cond": [
                          {"$in": ["$event", event_list2]},
                          1,
                          0
                      ]
                  }
              }
          }
      },
      doc! {
          "$match": {
              "e1": {"$gt": 0},
              "e2": {"$gt": 0}
          }
      },
      doc! {
          "$group": {
              "_id": "$pt_group",
              "ptids": {"$addToSet": "$_id"}
          }
      }
  ];

  // Execute the aggregation
  let mut cursor = db.eii_collection
      .aggregate(pipeline, None)
      .map_err(|_| Status::InternalServerError)?;

  // Convert the results to HashMap
  let mut candidates = HashMap::new();
  while let Some(result) = cursor.next() {
      let doc = result.map_err(|_| Status::InternalServerError)?;
      if let (Ok(group_id), Ok(ptids)) = (
          doc.get_i32("_id"),
          doc.get_array("ptids")
      ) {
          let ptid_list: Vec<String> = ptids
              .iter()
              .filter_map(|ptid| ptid.as_str().map(|s| s.to_string()))
              .collect();
          candidates.insert(group_id, ptid_list);
      }
  }

  Ok(Json(CandidateResponse { groups: candidates }))
}