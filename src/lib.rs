#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
#![allow(dead_code)]
#![allow(unused_variables)]
use dotenv::dotenv;
use log::info;
use tracing::Subscriber;



// init log config
pub fn init() {
    dotenv().ok();
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    // info!("INFO");
    // let _ = env_logger::builder()
    //     .target(env_logger::Target::Stdout)
    //     .filter_level(log::LevelFilter::Trace)
    //     .is_test(true)
    //     .try_init();
}

pub fn setup_subscribers() -> Box<impl Subscriber + Send + 'static> {
    let file = tracing_appender::rolling::hourly("./logs", "application.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file);

    let console = Layer::new()
        .with_writer(std::io::stdout)
        .pretty();

    let inspector = Layer::new()
        .with_writer(non_blocking)
        .json();

    Box::new(tracing_subscriber::registry().with(console).with(inspector))
}