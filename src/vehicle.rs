#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
#![allow(dead_code)]
#![allow(unused_variables)]
use std::collections::HashMap;

use axum::{
    debug_handler, extract::Query, http::StatusCode, routing::{get, post}, Json, Router
};
use log::info;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Vehicle {
    manufacturer: String,
    model: String,
    year: u32,
    id: Option<String>,
}

#[debug_handler]
pub async fn vehicle_get() -> Json<Vehicle> {
    info!("Caller retrieved a vehicle from auxm");
    Json::from(
        Vehicle {
            manufacturer: "Dodge".to_string(),
            model: "RAM 1500".to_string(),
            year: 2021,
            id: Some(uuid::Uuid::new_v4().to_string()),
        }
    )
}

// #[debug_handler]
// pub async fn vehicle_post(Json(mut v): Json<Vehicle>) -> Json<Vehicle> {
//     info!("Manufacturer: {}, model: {}, year: {}", v.manufacturer, v.model, v.year);
//     v.id = Some(Uuid::new_v4().to_string());

//     return Json::from(v);
// }

#[debug_handler]
pub async fn vehicle_post(Query(mut v): Query<Vehicle>) -> Json<Vehicle> {
    info!("Manufacturer: {}, model: {}, year: {}", v.manufacturer, v.model, v.year);
    v.id = Some(Uuid::new_v4().to_string());
    return Json::from(v);
}
