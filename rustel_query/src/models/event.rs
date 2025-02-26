use mongodb::bson;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use crate::{models::cde::CDE, models::tcde::TCDE};

// tel cde schema: id, cde, tcde, count

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: i32,
    // array of cde ids
    pub cde: Vec<i32>,
    pub tcde: i32,
    pub count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventRecord {
    pub ptid: String,
    pub event_id: i32,
    pub time: bson::DateTime
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventRecordDetail {
    pub time: bson::DateTime,
    pub cde: Vec<CDE>,
    pub tcde: TCDE,
}

#[derive(Debug, Serialize)]
pub struct EventDetailResponse {
    cde: Vec<CDE>,
    tcde: TCDE,
}