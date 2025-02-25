use crate::{models::cde::CDE, models::tcde::TCDE, database::mongodb::MongoRepo};
use mongodb::results::{self, InsertOneResult};
use rocket::{http::Status, serde::json::Json, State};
use mongodb::{options::ClientOptions, Client, bson::doc, options::FindOptions};
use mongodb::bson::Regex;

#[get("/cde/<path>")]
pub fn get_cde(db: &State<MongoRepo>, path: &str) -> Result<Json<CDE>, Status> {
    let id = path;
    if id.is_empty() {
        return Err(Status::BadRequest);
    };
    let id = id.parse::<i32>().unwrap();
    print!("id: {}", id);
    let filter = doc! {"id": id};
    let cde_detail: Option<CDE> = db
      .cde_collection
      .find_one(filter, None)
      .ok()
      .expect("Error getting event's detail");

    match cde_detail {
        Some(cde) => Ok(Json(cde)),
        None => Err(Status::InternalServerError),
    }
}

#[get("/tcde/<path>")]
pub fn get_tcde(db: &State<MongoRepo>, path: &str) -> Result<Json<TCDE>, Status> {
    let id = path;
    if id.is_empty() {
        return Err(Status::BadRequest);
    };
    let id = id.parse::<i32>().map_err(|_| Status::BadRequest)?;
    print!("id: {}", id);
    let filter = doc! {"id": id};
    let tcde_detail: Option<TCDE> = db
      .tcde_collection
      .find_one(filter, None)
      .ok()
      .expect("Error getting tcde's detail");

    match tcde_detail {
        Some(tcde) => Ok(Json(tcde)),
        None => Err(Status::InternalServerError),
    }
}

#[get("/search_cde?<collection>&<field>&<value>&<partial>&<limit>")]
pub fn search_cde(
  db: &State<MongoRepo>, 
  collection: Option<&str>,  
  field: Option<&str>,  
  value: &str,
  partial: Option<bool>,
  limit: Option<i64>,
) -> Result<Json<Vec<CDE>>, Status> {
  let mut filter = doc! {};
  // check if collection is provided
  if let Some(collection) = collection {
    filter.insert("collection", collection);
  }else {
    filter.insert("collection", doc!{ "$exists": true });
  }
  // check if field is provided
  if let Some(field) = field {
    filter.insert("field", field);
  }else {
    filter.insert("field", doc!{ "$exists": true });
  }
  // value to lower case
  let value = value.to_lowercase();
  if let Some(partial) = partial {
    if partial {
      filter.insert("str", doc! { "$regex": value });
    }else {
      filter.insert("str", value);
    }
  }else {
    filter.insert("str", value);
  }
  println!("filter: {:?}", filter);

  let mut find_options = FindOptions::builder().sort(doc! { "count": -1 }).build();
  if let Some(limit) = limit {
    find_options.limit = Some(limit);
  }
  let cursor = db
    .cde_collection
    .find(filter, find_options)
    .ok()
    .expect("Error getting event's detail");

    let mut results: Vec<CDE>= Vec::new();
    for result in cursor {
        match result {
            Ok(cde) => {
                results.push(cde);
            }
            Err(e) => {
                println!("Error getting cde: {:?}", e);
            }
        }
    }
    Ok(Json(results))
  
}
