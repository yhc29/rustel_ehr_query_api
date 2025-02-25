use mongodb::bson;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

// tel cde schema: id, cde, tcde, count

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: i32,
    // array of cde ids
    pub cde: Vec<i32>,
    pub tcde: i32,
    pub count: i32,
}