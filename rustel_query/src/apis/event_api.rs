use crate::{models::event::Event, models::cde::CDE, models::tcde::TCDE, database::mongodb::MongoRepo};
use mongodb::results::{self, InsertOneResult};
use rocket::{http::Status, serde::json::Json, State};
use mongodb::{options::ClientOptions, Client, bson::doc, options::FindOptions};
use mongodb::bson::Regex;
use serde::Serialize;
use rocket::request::FromParam;
use serde_json::{Value, json};
use percent_encoding::percent_decode_str;

use super::cde_api;

#[derive(Debug, Serialize)]
pub struct EventDetailResponse {
    pub cde: Vec<CDE>,
    pub tcde: TCDE,
}

// Add a wrapper struct for parsing
#[derive(Debug)]
pub struct CdeArrayParam(pub Vec<Vec<i32>>);

#[rocket::async_trait]
impl<'r> rocket::form::FromFormField<'r> for CdeArrayParam {
    fn from_value(field: rocket::form::ValueField<'r>) -> rocket::form::Result<'r, Self> {
        // Only decode URL once
        let decoded = percent_encoding::percent_decode_str(field.value)
            .decode_utf8()
            .map_err(|_| rocket::form::Error::validation("Failed to decode URL"))?
            .to_string();
            
        // Parse JSON once
        let parsed: Value = serde_json::from_str(&decoded)
            .map_err(|_| rocket::form::Error::validation("Failed to parse JSON"))?;
        
        let mut result = Vec::new();
        if let Value::Array(outer) = parsed {
            for inner in outer {
                if let Value::Array(numbers) = inner {
                    let inner_vec: Result<Vec<i32>, _> = numbers
                        .into_iter()
                        .map(|n| n.as_i64().map(|n| n as i32).ok_or("Invalid number"))
                        .collect();
                    result.push(inner_vec.map_err(|_| rocket::form::Error::validation("Invalid number"))?);
                } else {
                    return Err(rocket::form::Errors::from(rocket::form::Error::validation("Invalid inner array")));
                }
            }
        } else {
            return Err(rocket::form::Errors::from(rocket::form::Error::validation("Invalid outer array")));
        }
        
        Ok(CdeArrayParam(result))
    }
}

#[derive(Debug)]
pub struct StringArrayParam(pub Vec<String>);
#[rocket::async_trait]
impl<'r> rocket::form::FromFormField<'r> for StringArrayParam {
    fn from_value(field: rocket::form::ValueField<'r>) -> rocket::form::Result<'r, Self> {
        // First, try parsing as JSON
        let decoded = percent_decode_str(field.value)
            .decode_utf8()
            .map_err(|_| rocket::form::Error::validation("Failed to decode URL"))?
            .to_string();
        
        // Try to parse as JSON first
        if let Ok(parsed) = serde_json::from_str::<Value>(&decoded) {
            if let Value::Array(outer) = parsed {
                let mut result = Vec::new();
                for inner in outer {
                    if let Value::String(s) = inner {
                        result.push(s);
                    } else if let Value::Number(n) = inner {
                        result.push(n.to_string());
                    } else {
                        return Err(rocket::form::Errors::from(rocket::form::Error::validation("Invalid array element")));
                    }
                }
                return Ok(StringArrayParam(result));
            }
        }
        
        // If JSON parsing fails, try comma-separated format
        let values: Vec<String> = decoded
            .trim_matches(|c| c == '[' || c == ']')
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        if values.is_empty() {
            return Err(rocket::form::Errors::from(rocket::form::Error::validation("Empty array")));
        }
        
        Ok(StringArrayParam(values))
    }
}

#[get("/event/<path>")]
pub fn get_event(db: &State<MongoRepo>, path: &str) -> Result<Json<Event>, Status> {
    let id = path;
    if id.is_empty() {
        return Err(Status::BadRequest);
    };
    let id = id.parse::<i32>().unwrap();
    print!("id: {}", id);
    let filter = doc! {"id": id};
    let event_doc: Option<Event> = db
      .event_collection
      .find_one(filter, None)
      .ok()
      .expect("Error getting event's detail");

    match event_doc {
        Some(event) => Ok(Json(event)),
        None => Err(Status::InternalServerError),
    }
}

#[get("/event_detail/<path>")]
pub fn get_event_detail(db: &State<MongoRepo>, path: &str) -> Result<Json<EventDetailResponse>, Status> {
    let id = path;
    if id.is_empty() {
        return Err(Status::BadRequest);
    };
    let id = id.parse::<i32>().unwrap();
    // print!("id: {}", id);
    let filter = doc! {"id": id};
    let event_doc: Option<Event> = db
      .event_collection
      .find_one(filter, None)
      .ok()
      .expect("Error getting event's detail");

    if event_doc.is_none() {
      return Err(Status::NotFound);
    }

    // get cde detail list 
    let event = event_doc.unwrap();
    let cde_list = event.cde;
    let tcde = event.tcde;
    let mut cde_results = Vec::new();
    let mut tcde_result =  None;
    for cde in cde_list {
      match cde_api::get_cde(db, cde.to_string().as_str()) {
        Ok(Json(cde)) => {
          cde_results.push(cde);
        }
        Err(_) => continue,
      }
    }
    // get tcde detail
    match cde_api::get_tcde(db, tcde.to_string().as_str()) {
      Ok(Json(tcde)) => {
        tcde_result = Some(tcde);
      }
      Err(_) => {},
    }
    Ok(Json(EventDetailResponse {
      cde: cde_results,
      tcde: tcde_result.unwrap(),
    }))
}

// gives a event id and returns the event summary of key-value json
#[get("/event_summary/<path>")]
pub fn get_event_summary(db: &State<MongoRepo>, path: &str) -> Result<Json<Value>, Status> {
    let id = path;
    // get event detail
    let event_detail = get_event_detail(db, id).map_err(|_| Status::InternalServerError)?.into_inner();
    let cde_list = event_detail.cde;
    let tcde = event_detail.tcde;
    
    let mut result = json!({
      "time_field": {tcde.collection: tcde.field}
    });
    for cde in cde_list {
      // if cde.collection not in result, add it
      if !result.as_object().unwrap().contains_key(&cde.collection) {
        result[&cde.collection] = json!({});
      }
      // add cde.field and value to cde.collection
      result[&cde.collection][&cde.field] = json!(cde.value);
    }

    Ok(Json(result))
}

#[get("/search_events?<cde>&<tcde>")]
pub fn search_events(
  db: &State<MongoRepo>, 
  cde: Option<CdeArrayParam>,  
  tcde: Option<i32>,  
) -> Result<Json<Vec<i32>>, Status> {
  // check if either cde or tcde is provided, return error if not
  if cde.is_none() && tcde.is_none() {
    return Err(Status::BadRequest);
  }
  println!("cde: {:?}", cde);
  // check if cde is provided, if not, only search by tcde
  let mut and_stmt = vec![];
  if let Some(CdeArrayParam(cde_lists)) = cde {
    for cde_list in cde_lists {
      and_stmt.push(doc! {
        "cde": {
          "$in": cde_list
        }
      });
    }
  }
  if let Some(tcde) = tcde {
    and_stmt.push(doc! {
      "tcde": tcde
    });
  }
  let filter = doc! {
    "$and": and_stmt
  };

  println!("filter: {:?}", filter);

  // create aggregation pipeline to get a list of event ids
  let pipeline = vec![
    doc! {
      "$match": filter
    },
    doc! {
      "$group": {
        "_id": Option::<i32>::None,
        "events": {
          "$push": "$id"
        }
      }
    }
  ];
  let cursor = db
    .event_collection
    .aggregate(pipeline, None)
    .ok()
    .expect("Error getting event's detail");

  let mut results = Vec::new();
  for result in cursor {
    match result {
      Ok(doc) => {
        let events = doc.get_array("events").unwrap();
        for event in events {
          results.push(event.as_i32().unwrap());
        }
      }
      Err(_) => return Err(Status::InternalServerError),
    }
  }
  Ok(Json(results))
}

#[get("/search_events_by_omop?<omop_concepts>")]
pub fn search_events_by_omop(
  db: &State<MongoRepo>, 
  omop_concepts: StringArrayParam
) -> Result<Json<Vec<i32>>, Status> {
  // get cde id list from mapping
  let mut cde_list = Vec::new();
  let filter = doc! {
    "omop_concept_id": {
      "$in": &omop_concepts.0
    }
  };
  println!("filter: {:?}", filter);
  
  let cursor = db
    .omop_mapping_collection
    .find(filter, None)
    .ok()
    .expect("Error getting event's detail");
  for result in cursor {
    match result {
      Ok(doc) => {
        let cde_id = doc.cde_id;
        cde_list.push(cde_id);
      }
      Err(_) => return Err(Status::InternalServerError),
    }
  }
  // search events by cde list using search_events
  let results = search_events(db, Some(CdeArrayParam(vec![cde_list])), None)
    .map_err(|_| Status::InternalServerError)?
    .into_inner();
  Ok(Json(results))

}