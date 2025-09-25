#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
#![allow(dead_code)]
#![allow(unused_variables)]
use dotenv::dotenv;
use log::info;
use tracing::Subscriber;
use flexi_logger::{
    Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming, WriteMode,
};

pub mod box_rc_test;

pub mod vehicle;

pub mod flexible_test;

pub mod redis_test;

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
     // 初始化 flexi_logger
    // let logger = Logger::try_with_env_or_str("info, my_app::critical=trace").unwrap()
    //     // 输出到文件：路径为 ./logs/my_app.log
    //     .log_to_file(
    //         FileSpec::default()
    //             .directory("logs")
    //             .basename("my_app")
    //             .suppress_timestamp(), // 无时间戳后缀，便于旋转
    //     )
    //     // 启用异步缓冲模式：缓冲容量 8KB，每 1s flush
    //     .write_mode(WriteMode::BufferAndFlush)
    //     // 文件旋转：每天旋转一次
    //     .rotate(
    //         Criterion::Age(Age::Day), // 触发条件：每日
    //         Naming::Timestamps,       // 新文件命名：添加时间戳
    //         Cleanup::KeepCompressedFiles(30), // 保留 30 个压缩旧文件
    //     )
    //     // 复制 ERROR 级别日志到 stderr
    //     .duplicate_to_stderr(Duplicate::Error)
    //     // 支持规格文件：实时修改日志级别
    //     .start_with_specfile("logspec.toml")
    //     .unwrap();
        // 启动日志
        // .start()?;

    // 获取 LoggerHandle 以便后续动态调整
    // let _handle = logger;
}

// pub fn setup_subscribers() -> Box<impl Subscriber + Send + 'static> {
//     let file = tracing_appender::rolling::hourly("./logs", "application.log");
//     let (non_blocking, _guard) = tracing_appender::non_blocking(file);

//     let console = Layer::new()
//         .with_writer(std::io::stdout)
//         .pretty();

//     let inspector = Layer::new()
//         .with_writer(non_blocking)
//         .json();

//     Box::new(tracing_subscriber::registry().with(console).with(inspector))
// }