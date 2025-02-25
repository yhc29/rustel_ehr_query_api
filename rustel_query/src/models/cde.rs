use mongodb::bson;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CDEValue {
    String(String),
    Integer(i32),
    Float(f64),
}

// tel cde schema: id, collection, field, value, value_type, str, count

#[derive(Debug, Serialize, Deserialize)]
pub struct CDE {
    pub id: i32,
    pub collection: String,
    pub field: String,
    pub value: CDEValue,
    pub value_type: String,
    #[serde(rename = "str")]
    pub value_str: String,
    pub count: i32,
}