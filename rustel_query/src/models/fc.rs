use mongodb::bson;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

// tel fc schema: ptid, pt_group, event1, indices, event2, fc_date_diffs

#[derive(Debug, Serialize, Deserialize)]
pub struct FC {
    pub ptid: String,
    pub pt_group: i32,
    pub event1: Option<i32>,
    // array of indices
    pub indices:  Option<Vec<i32>>,
    pub event2:  Option<i32>,
    // array of fc_date_diffs in double
    pub fc_date_diffs:  Option<Vec<f64>>,
}