use mongodb::bson;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

// tel cde schema: id, collection, field, type, count

#[derive(Debug, Serialize, Deserialize)]
pub struct TCDE {
    pub id: i32,
    pub collection: String,
    pub field: String,
    pub value_type: String,
    pub count: i32,
}