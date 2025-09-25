#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::time::Duration;
use deadpool_postgres::{tokio_postgres::NoTls, Config, Manager, ManagerConfig, Pool, RecyclingMethod};
use futures::channel::oneshot;
use log::info;
use mini_redis::client;
use vehicle_manager_axum::{init};
use tokio_stream::StreamExt;

#[tokio::test]
async fn it_img_inverted_test() {
    init();

    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.port = Some(5432);
    cfg.dbname = Some("postgres".to_string());
    cfg.user = Some("postgres".to_string());
    cfg.password = Some("123456".to_string());
    // 配置超时时间
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool = cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1), NoTls).unwrap();

    // 监控连接池状态
    // tokio::spawn(async move {
    //     loop {
    //         tokio::time::sleep(Duration::from_secs(10)).await;
    //         let status = pool.clone().status();
    //         info!(
    //             "Pool status: active={}, idle={}, size={}",
    //             status.available, status.max_size, status.size
    //         );
    //     }
    // });
    

    let client = pool.get().await.unwrap();

    let rows = client.query("select * from users ", &[]).await.unwrap();
    for row in rows {
        let id: i32 = row.get(0);
        let firstname:&str = row.get(1);
        info!("id: {}, name: {}", id, firstname);
    }

}


#[tokio::test]
async fn it_select_test01() {
    init();
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();

    tokio::spawn(async {
        let _ = tx1.send("one");
    });

    tokio::spawn(async {
        let _ = tx2.send("two");
    });

    tokio::select! {
        val = rx1 => {
            info!("rx1 completed first with {:?}", val);
        }
        val = rx2 => {
            info!("rx2 completed first with {:?}", val);
        }
    }
}

#[tokio::test]
async fn it_stream_test01() {
    init();
    let mut stream = tokio_stream::iter(&[1,2,3]);

    while let Some(v) = stream.next().await {
        info!("Got = {:?}", v);
    }
}

async fn publish() -> mini_redis::Result<()> {
    let mut client = client::connect("127.0.0.1:6379").await?;

    let topic = "number";
    client.publish(topic, "1".into()).await?;
    client.publish(topic, "two".into()).await?;
    client.publish(topic, "3".into()).await?;
    client.publish(topic, "four".into()).await?;
    client.publish(topic, "five".into()).await?;
    client.publish(topic, "6".into()).await?;
    Ok(())
}

async fn subscriber() -> mini_redis::Result<()> {
    let client = client::connect("127.0.0.1:6379").await?;
    let subscriber = client.subscribe(vec!["numbers".to_string()]).await?;
    let messages = subscriber.into_stream()
        .filter_map(|msg| match msg {
            Ok(msg) if msg.content.len() == 1 => Some(msg.content),
            _ => None,
        })
        .take(3);

    tokio::pin!(messages);

    while let Some(msg) = messages.next().await {
        info!("Got = {:?}", msg);
    }
    Ok(())
}

#[tokio::test]
async fn it_publish_test() -> mini_redis::Result<()>{
    init();
    tokio::spawn(async {
        publish().await
    });

    subscriber().await?;

    info!("Done");
    Ok(())
}