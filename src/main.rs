#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
#![allow(dead_code)]
#![allow(unused_variables)]

use log::info;


fn main() {
    vehicle_manager_axum::init();

    info!("Hello, world!");
}
