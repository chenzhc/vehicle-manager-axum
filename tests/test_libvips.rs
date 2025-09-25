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
use rand::{distr::{Distribution, SampleString, Uniform}, Rng};
use rand_distr::{Alphanumeric, Normal, StandardUniform};
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

#[test]
fn it_rng_test01() {
    init();
    let mut rng = rand::rng();

    let n1: u8 = rng.random();
    let n2: u16 = rng.random();

    info!("u8 : {}", n1);
    info!("u16 : {}", n2);
    info!("u32: {}", rng.random::<u32>());
    info!("i32: {}", rng.random::<i32>());
    info!("float: {}", rng.random::<f64>());
    
    info!("{}", rng.random_range(0..10));
    info!("{}", rng.random_range(0.0..10.0));
}

#[test]
fn it_rng_test02() {
    init();
    let mut rng = rand::rng();
    let die: Uniform<u16> = Uniform::try_from(1..7).unwrap();

    loop {
        let throw = die.sample(&mut rng);

        info!(": {}", throw);
        if throw == 6 {
            break;
        }
    }
}

#[test]
fn it_rng_test03() {
    init();
    let mut rng = rand::rng();
    // 正态分布
    let normal = Normal::new(2.0,9.0).unwrap();
    let v = normal.sample(&mut rng);
    info!("正态分布: {}", v);

    let rand_tuple = rng.random::<(i32, bool, f64)>();
    let rand_point: Point = rng.random();

    info!("随机值元组: {:?}", rand_tuple);
    info!("随机值结构体: {:?}", rand_point);
}

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl Distribution<Point> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Point {
        let (rand_x, rand_y) = rng.random();
        Point {
            x: rand_x,
            y: rand_y,
        }
    }
}

// 从一组字母数字生成一个随机值
#[test]
fn it_rng_test04() {
    init();
    let rand_string: String = Alphanumeric.sample_string(&mut rand::rng(), 30);
    
    // 随机密码: t76Rapb7IKh3153UROq1vSYpWPP46v
    info!("随机密码: {}", rand_string);

}