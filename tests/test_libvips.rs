#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::{fs::File, io::Write, thread, time::Duration};
use crossbeam_channel::{bounded, unbounded};
use deadpool_postgres::{tokio_postgres::NoTls, Config, Manager, ManagerConfig, Pool, RecyclingMethod};
use flate2::{write::GzEncoder, read::GzDecoder, Compression};
use futures::channel::oneshot;
use log::info;
use log4rs::append::rolling_file::policy;
use mini_redis::client;
use rand::{distr::{Distribution, SampleString, Uniform}, Rng};
use rand_distr::{Alphanumeric, Normal, StandardUniform};
use tar::{Archive, Builder};
use vehicle_manager_axum::{init};
use tokio_stream::StreamExt;
use zip::{write::FileOptions, ZipWriter};
extern crate tar;
extern crate zip;

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

// 你想设定一组自定义字符，然后从此自定义字符范围内生成一个随机值，
// 比如创建只包含小写字母、数字，以及特殊字符的随机密码
#[test]
fn it_rng_test05() {
    init();
    const CHARSET: &[u8] =  b"abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
    const PASSWORD_LEN: usize = 30;

    let mut rng = rand::rng();

    let password: String = (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    info!("随机密码: {}", password);

}

#[test]
fn it_vec_sort_test01() {
    init();
    let mut vec = vec![1,5,10,2,15];
    info!("排序前: {:?}", vec);

    vec.sort();
    info!("排序后: {:?}", vec);

    let mut vec = vec![1.1, 1.15, 5.5, 1.123, 2.0];
    info!("排序前: {:?}", vec);

    vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
    info!("排序后: {:?}", vec);
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Person {
    name: String, 
    age: u32,
}

impl Person {
    pub fn new(name: &str, age: u32) -> Self {
        Person { name: name.to_string(), age: age }
    }
}

#[test]
fn it_person_eq_test01() {
    init();
    let mut people = vec![
        Person::new("Zhang", 25),
        Person::new("Liu", 60),
        Person::new("Wang", 1),
    ];
    info!("排序前: {:?}", people);

    people.sort();
    info!("排序后: {:?}", people);

    people.sort_by(|a, b| b.age.cmp(&a.age));
    info!("排序后(age): {:?}", people);

}

#[test]
fn it_tar_gz_test01() -> anyhow::Result<()> {
    init();
    let path = "old_file.tar.gz";

    let tar_gz = File::open(path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(".")?;

    Ok(())
}


#[test]
fn it_tar_gz_dir_test01() -> anyhow::Result<()> {
    init();
    let mut buffer = Vec::new();
    let mut encoder = GzEncoder::new(&mut buffer, Compression::default());
    encoder.write_all(b"your_data_to_compose")?;
    encoder.flush()?;

    let file = File::create("archive.tar").unwrap();
    let mut builder = Builder::new(file);
    builder.append_file("1.txt", &mut File::open("./files/1.txt").unwrap()).unwrap();

    Ok(())
}

#[test]
fn it_un_tar_file_test01() {
    init();

    let file = File::open("archive.tar").unwrap();
    let mut archive = Archive::new(file);
    archive.unpack("./test/").unwrap();

}

#[test]
fn it_zip_file_test01() -> anyhow::Result<()> {
    init();
    let file = File::create("archive.tar.gz").unwrap();
    let enc = GzEncoder::new(file, Compression::default());
    let mut tar_file = tar::Builder::new(enc);
    tar_file.append_dir_all(".", "./files/1.txt")?;

    Ok(())
}

// 折半查找 
fn find_max(arr: &[i32]) -> Option<i32> {
    const THRSHOLD: usize = 2;

    if arr.len() <= THRSHOLD {
        return arr.iter().cloned().max();
    }

    let mid = arr.len() / 2;
    let (left, right) = arr.split_at(mid);

    crossbeam::scope(|s| {
        let thread_l = s.spawn(|_| find_max(left));
        let thread_r = s.spawn(|_| find_max(right));

        let max_l = thread_l.join().unwrap()?;
        let max_r = thread_r.join().unwrap()?;

        Some(max_l.max(max_r))
    }).unwrap()
}

#[test]
fn it_find_max_test01() {
    init();
    let arr = &[1, 25, -4, 10];
    let max = find_max(arr);
    info!("{:?}", max);

}

#[test]
fn it_channel_test04() {
    init();

    let (snd1, recv1) = bounded(1);
    let (snd2, recv2) = bounded(2);
    let n_msgs = 4;
    let n_workers = 2;

    crossbeam::scope(|s| {
        // 生产者线程
        s.spawn(|_| {
            for i in 0..n_msgs {
                snd1.send(i).unwrap();
                info!("Source sent {}", i);
            }
            // 关闭信道这是退出的必要条件
            drop(snd1);
        });

        // 由2个线程并行处理
        for _ in 0..n_workers {
            // 从数据源发送数据到接收器， 接收器接收数据 
            let (sendr, recvr) = (snd2.clone(), recv1.clone());
            // 在不同的线程中处理数据 
            s.spawn(move |_| {
                thread::sleep(Duration::from_millis(500));
                // 接收数据， 直到信道关闭前
                for msg in recvr.iter() {
                    info!("Worker {:?} received: {}.", thread::current().id(), msg);
                    sendr.send(msg*2).unwrap();
                }
            });
        }
        // 关闭信道，否则接收器不会关闭
        drop(snd2);

        // 接收数据 
        for msg in recv2.iter() {
            info!("Sink received {}", msg);
        }
    }).unwrap();
}

// 两个线程间通信
#[test]
fn it_channel_test05() {
    init();
    let (tx, rx) = unbounded();
    let n_msgs = 5;
    crossbeam::scope(|s| {
        s.spawn(|_| {
            for i in 0..n_msgs {
                tx.send(i).unwrap();
                thread::sleep(Duration::from_millis(100));
            }
        });
    }).unwrap();

    for _ in 0..n_msgs {
        let msg = rx.recv().unwrap();
        info!("Received: {}", msg);
    }
}