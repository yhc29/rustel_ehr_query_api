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

use crate::apis::event_api::{search_events_by_omop,StringArrayParam};

#[derive(Debug)]
pub struct EventListParam(pub Vec<i32>);

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
  // count for number of all candidates, ptids list for each group
  count: i32,
  groups: HashMap<i32, Vec<String>>
}
impl IntoIterator for CandidateResponse {
    type Item = (i32, Vec<String>);
    type IntoIter = std::collections::hash_map::IntoIter<i32, Vec<String>>;

    fn into_iter(self) -> Self::IntoIter {
        self.groups.into_iter()
    }
}

#[get("/eii_and?<event_list1>&<event_list2>")]
pub async fn eii_and(
  db: &State<MongoRepo>,
  event_list1: EventListParam,
  event_list2: EventListParam
) -> Result<Json<CandidateResponse>, Status> {
  let param1: Vec<i32> = event_list1.0;
  let param2: Vec<i32> = event_list2.0;
  // Combine both event lists for the $in operator
  let combined_events = param1.iter().chain(&param2).cloned().collect::<Vec<_>>();
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
                          {"$in": ["$event", param1]},
                          1,
                          0
                      ]
                  }
              },
              "e2": {
                  "$sum": {
                      "$cond": [
                          {"$in": ["$event", param2]},
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
  let mut count = 0;
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
          candidates.insert(group_id, ptid_list.clone());
          count += ptid_list.len() as i32;
      }
  }

  Ok(Json(CandidateResponse { count, groups: candidates }))
}

#[get("/eii_and_omop?<omop_concept_id_list1>&<omop_concept_id_list2>")]
pub async fn eii_and_omop(
  db: &State<MongoRepo>,
  omop_concept_id_list1: StringArrayParam,
  omop_concept_id_list2: StringArrayParam
) -> Result<Json<CandidateResponse>, Status> {
  // Get event lists using search_events_by_omop
  let event_list1 = search_events_by_omop(db, omop_concept_id_list1)
      .map_err(|_| Status::InternalServerError)?
      .into_inner(); // Extract Vec<i32> from Json<Vec<i32>>
  let event_list2 = search_events_by_omop(db, omop_concept_id_list2)
      .map_err(|_| Status::InternalServerError)?
      .into_inner(); // Extract Vec<i32> from Json<Vec<i32>>
  // convert event_list to EventListParam
  let event_param1 = EventListParam(event_list1);
  let event_param2 = EventListParam(event_list2);
  // call eii_and with event_list input
  let result = eii_and(db, event_param1, event_param2).await?;
  Ok(result)

}
  

