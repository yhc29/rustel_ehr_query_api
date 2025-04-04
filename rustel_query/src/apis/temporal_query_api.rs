use crate::{models::event::Event, models::cde::CDE, models::tcde::TCDE, database::mongodb::MongoRepo};
use mongodb::change_stream::event;
use mongodb::results::{self, InsertOneResult};
use futures::stream::StreamExt;
use rocket::{http::Status, serde::json::Json, State};
use mongodb::{options::ClientOptions, Client, bson::doc, options::FindOptions};
use mongodb::bson::Regex;
use serde::Serialize;
use rocket::request::FromParam;
use serde_json::Value;
use percent_encoding::percent_decode_str;

use std::collections::HashMap;

use crate::apis::event_api::{search_events_by_omop,StringArrayParam};
use super::eii_api;

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
pub struct TemporalQueryResponse {
  count: i32,
  ptids: Vec<String>
}

#[get("/efcfcd_diamond?<event_list1>&<event_list2>&<delta_max>&<delta_max_op>&<cooccurrence>&<negation>")]
pub async fn efcfcd_diamond_v4_1(
    db: &State<MongoRepo>,
    event_list1: EventListParam,
    event_list2: EventListParam,
    delta_max: i64,
    delta_max_op: Option<String>,
    cooccurrence: Option<bool>,
    negation: Option<bool>
) -> Result<Json<TemporalQueryResponse>, Status> {
    // Get default values
    let delta_max_op = delta_max_op.unwrap_or_else(|| "lt".to_string());
    let cooccurrence = cooccurrence.unwrap_or(true);
    let negation = negation.unwrap_or(false);
    
    // Get candidates from eii
    let mut result = Vec::new();
    let event_list1_eii = eii_api::EventListParam(event_list1.0.clone());
    let event_list2_eii = eii_api::EventListParam(event_list2.0.clone());
    let candidates = match eii_api::eii_and(db, event_list1_eii, event_list2_eii).await {
        Ok(Json(candidate_response)) => candidate_response,
        Err(status) => return Err(status),
    };
    
    // Determine delta_min_op based on cooccurrence
    let delta_min_op = if cooccurrence { "gte" } else { "gt" };
    
    // Build match statement based on negation
    let match_stmt = if negation {
        doc! {
            "$match": {
                "e2_fc": {
                    "$not": {
                        "$elemMatch": {
                            format!("${}", delta_max_op): delta_max
                        }
                    }
                }
            }
        }
    } else {
        doc! {
            "$match": {
                "e2_fc": {
                    "$elemMatch": {
                        format!("${}", delta_min_op): 0,
                        format!("${}", delta_max_op): delta_max
                    }
                }
            }
        }
    };

    // Process each candidate group
    for (pt_group_id, candidates_list) in candidates.into_iter() {
      println!("pt_group_id: {:?}", pt_group_id);
      let candidates_num = candidates_list.len();
      println!("candidates_num: {:?}", candidates_num);
      // split the candidates into smaller groups with max 5000 elements
      let mut candidates_list_split = Vec::new();
      let mut candidates_list_temp = Vec::new();
      for candidate in candidates_list {
        candidates_list_temp.push(candidate);
        if candidates_list_temp.len() == 5000 {
          candidates_list_split.push(candidates_list_temp);
          candidates_list_temp = Vec::new();
        }
      }
      if !candidates_list_temp.is_empty() {
        candidates_list_split.push(candidates_list_temp);
      }
      for ptid_list in candidates_list_split {
        // println!("ptid_list count for query: {:?}", ptid_list.len());
        let pipeline = vec![
            doc! {
                "$match": {
                    "$or": [
                        {
                            "ptid": {"$in": &ptid_list},
                            "event1": {"$in": &event_list1.0.clone()}
                        },
                        {
                            "ptid": {"$in": &ptid_list},
                            "event2": {"$in": &event_list2.0.clone()}
                        }
                    ]
                }
            },
            doc! {
                "$group": {
                    "_id": "$ptid",
                    "e1_i": {
                        "$push": {
                            "$cond": [
                                {"$ne": ["$event1", null]},
                                "$indices",
                                []
                            ]
                        }
                    },
                    "e2_fc": {
                        "$push": {
                            "$cond": [
                                {"$ne": ["$event2", null]},
                                "$fc_date_diffs",
                                null
                            ]
                        }
                    }
                }
            },
            doc! {
                "$project": {
                    "e1_i": {
                        "$reduce": {
                            "input": "$e1_i",
                            "initialValue": [],
                            "in": {"$concatArrays": ["$$value", "$$this"]}
                        }
                    },
                    "e2_fc": {
                        "$filter": {
                            "input": "$e2_fc",
                            "as": "item",
                            "cond": {"$ne": ["$$item", null]}
                        }
                    }
                }
            },
            doc! {
                "$match": {
                    "e1_i": {"$ne": []},
                    "e2_fc": {"$ne": []}
                }
            },
            doc! {
                "$project": {
                    "_id": 1,
                    "e2_fc": {
                        "$map": {
                            "input": "$e1_i",
                            "as": "index",
                            "in": {
                                "$min": {
                                    "$map": {
                                        "input": "$e2_fc",
                                        "as": "fc",
                                        "in": {"$arrayElemAt": ["$$fc", "$$index"]}
                                    }
                                }
                            }
                        }
                    }
                }
            },
            match_stmt.clone(),
            doc! {
                "$group": {
                    "_id": null,
                    "ptids": {"$addToSet": "$_id"}
                }
            }
        ];

        let mut cursor = db.fc_collection
            .aggregate(pipeline, None)
            .map_err(|_| Status::InternalServerError)?;
        while let Some(doc) = cursor
            .next()
            .transpose()
            .map_err(|_| Status::InternalServerError)? {
            if let Ok(ptids) = doc.get_array("ptids") {
                for ptid in ptids {
                    if let Some(ptid_str) = ptid.as_str() {
                        result.push(ptid_str.to_string());
                    }
                }
            }
        }
      }
    }

    Ok(Json(TemporalQueryResponse { 
      count: result.len() as i32,
      ptids: result 
    }))
}

#[get("/efcfcd_existential_cooccurrence?<event_list1>&<event_list2>&<delta_max>&<left_open>&<right_open>")]
pub async fn efcfcd_existential_cooccurrence(
  db: &State<MongoRepo>,
  event_list1: EventListParam,
  event_list2: EventListParam,
  delta_max: i64,
  left_open: bool,
  right_open: bool
) -> Result<Json<TemporalQueryResponse>, Status> {
  let delta_max_op = if right_open { "lt" } else { "lte" };
  let cooccurance = if left_open { false } else { true };
  let negation = false;

  let result = efcfcd_diamond_v4_1(db, event_list1, event_list2, delta_max, Some(delta_max_op.to_string()), Some(cooccurance), Some(negation)).await?;
  Ok(result)

}

#[get("/efcfcd_existential_cooccurrence_omop?<omop_concept_id_list1>&<omop_concept_id_list2>&<delta_max>&<left_open>&<right_open>")]
pub async fn efcfcd_existential_cooccurrence_omop(
  db: &State<MongoRepo>,
  omop_concept_id_list1: StringArrayParam,
  omop_concept_id_list2: StringArrayParam,
  delta_max: i64,
  left_open: bool,
  right_open: bool
) -> Result<Json<TemporalQueryResponse>, Status> {
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

    let delta_max_op = if right_open { "lt" } else { "lte" };
    let cooccurance = if left_open { false } else { true };
    let negation = false;

    // call efcfcd_diamond_v4_1 with event_list input
    let result = efcfcd_diamond_v4_1(db, event_param1, event_param2, delta_max, Some(delta_max_op.to_string()), Some(cooccurance), Some(negation)).await?;
    Ok(result)
}