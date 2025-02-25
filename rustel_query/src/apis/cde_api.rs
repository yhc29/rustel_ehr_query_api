use crate::{models::cde::CDE, database::mongodb::MongoRepo};
use mongodb::results::InsertOneResult;
use rocket::{http::Status, serde::json::Json, State};
use mongodb::{options::ClientOptions, Client, bson::doc, options::FindOptions};
use mongodb::bson::Regex;

#[get("/cde/<path>")]
pub fn get_cde(db: &State<MongoRepo>, path: &str) -> Result<Json<CDE>, Status> {
    let id = path;
    if id.is_empty() {
        return Err(Status::BadRequest);
    };
    let cde_detail = db.get_cde_by_id(&id);
    match cde_detail {
        Ok(cde) => Ok(Json(cde)),
        Err(_) => Err(Status::InternalServerError),
    }
}