#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::net::SocketAddr;
use log::info;
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::{instrument::WithSubscriber, metadata::LevelFilter, subscriber, Level, Subscriber};
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{
    filter::filter_fn, fmt, layer::Filter, prelude::*, registry::LookupSpan, EnvFilter, Registry
};

use vehicle_manager_axum::vehicle;

#[tokio::main]
async fn main() {
    vehicle_manager_axum::init();

    // 1, Create the axum router
    let app = Router::new()
        .route("/", get(|| async { "Hello, world!"}))
        .route("/vehicle", 
            get(vehicle::vehicle_get)
            .post(vehicle::vehicle_post)
        );
        // .layer(TraceLayer::new_for_http());

    // 2, Define the IP and port listener (TCP)
    let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
    info!("Starting server on {}", addr);


    // 3, axum server to lunch the web server 
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    
}
