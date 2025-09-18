#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::net::SocketAddr;

use axum::{routing::get, Router};
use log::info;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    vehicle_manager_axum::init();
    let file = tracing_appender::rolling::hourly("./logs", "application.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file);
    let console = Layer::new()
        .with_writer(std::io::stdout)
        .pretty();

    let inspector = Layer::new()
        .with_writer(non_blocking)
        .json();

    let subscriber = Box::new(tracing_subscriber::registry().with(console).with(inspector));

    // From here, the implementations are the same. They both use the subscriber varaible
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set a global collector");
    

    let app = Router::new()
        .route("/", get(|| async { "Hello, world!"}))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
    info!("Starting server on {}", addr);


    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    
}
