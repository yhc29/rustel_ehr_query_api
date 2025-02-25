use mongodb::bson;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

// tel eii schema: pt_group, event, ptids

#[derive(Debug, Serialize, Deserialize)]
pub struct Eii {
    pub pt_group: i32,
    pub event: i32,
    // array of ptids
    pub ptids: Vec<String>,
}