#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
#![allow(dead_code)]
#![allow(unused_variables)]

use core::num;
use std::{fs::{create_dir_all, File}, io::{BufReader, Read, Write}, num::NonZeroU32, path::Path, sync::{mpsc::channel, Mutex}, thread, time::Duration};
use anyhow::Error;
use crossbeam_channel::{bounded, unbounded};
use data_encoding::HEXUPPER;
use deadpool_postgres::{tokio_postgres::NoTls, Config, Manager, ManagerConfig, Pool, RecyclingMethod};
use flate2::{write::GzEncoder, read::GzDecoder, Compression};
use futures::channel::oneshot;
use glob::{glob_with, MatchOptions};
use log::info;
use log4rs::append::rolling_file::policy;
use mini_redis::client;
use rand::{distr::{Distribution, SampleString, Uniform}, Rng};
use rand_distr::{Alphanumeric, Normal, StandardUniform};
use rayon::{iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator}, slice::ParallelSliceMut};
use ring::{digest::{self, Context, Digest, SHA256}, error::Unspecified, hmac, pbkdf2, rand::SecureRandom};
use tar::{Archive, Builder};
use threadpool::ThreadPool;
use vehicle_manager_axum::{init};
use tokio_stream::StreamExt;
use walkdir::WalkDir;
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

lazy_static::lazy_static! {
    static ref FRUIT: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

fn insert(fruit: &str) -> anyhow::Result<()> {
    let mut db = FRUIT.lock().map_err(|_| "Failed to acquire MutexGuard").unwrap();
    db.push(fruit.to_string());
    Ok(())
}

#[test]
fn it_fruit_insert_test01() -> anyhow::Result<()> {
    init();
    insert("apple")?;
    insert("orange")?;
    insert("peach")?;
    {
        let db = FRUIT.lock().map_err(|_| "Failed to acquire MutexGuard").unwrap();
        db.iter().enumerate().for_each(|(i, item)| info!("{}: {}", i, item));
    }
    insert("grape")?;
    Ok(())
}

fn is_exe(entry: &Path, ext: &str) -> bool {
    match entry.extension() {
        Some(e) if e.to_string_lossy().to_lowercase() == ext => true,
        _ => false,
    }
}

fn compute_digest<P: AsRef<Path>>(filepath: P) -> Result<(Digest, P), Error> {
    let mut buf_reader = BufReader::new(File::open(&filepath)?);
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = buf_reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok((context.finish(), filepath))    
}

#[test]
fn it_is_exe_test01() {
    init();
    let str = "C:/Users/user/下载/ETax.exe";
    let path = Path::new(str);
    let rs = is_exe(path, "exe");
    info!("{}", rs);
    let digest_rs = compute_digest(path.to_owned()).unwrap();
    info!("{:?}", digest_rs);
}

#[test]
fn it_file_diegest_test01() -> anyhow::Result<()> {
    init();
    info!("{:?}", num_cpus::get());
    let pool = ThreadPool::new(num_cpus::get());
    info!("{:?}", pool);
    let (tx, rx) = std::sync::mpsc::channel();

    let root_path = "C:\\Users\\user\\.cargo\\bin";
    let walk_iter = WalkDir::new(root_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.path().is_dir() && is_exe(e.path(), "exe"));
  
    for entry in  walk_iter {
        let path = entry.path().to_owned();
        let tx = tx.clone();
        pool.execute(move || {
            let digest = compute_digest(path);
            tx.send(digest).expect("Could not send data!");
        });
    }

    drop(tx);
    for t in rx.iter() {
        let (sha, path) = t?;
        info!("{:?}, {:?}", sha, path);
    }

    Ok(())
}

#[test]
fn it_rayon_test01() {
    init();
    let mut arr = [0,7,9,11];
    arr.par_iter_mut().for_each(|p| *p = -1);
    info!("{:?}", arr);

    let mut vec = vec![2,4,6,8];

    info!("{:?}", vec.par_iter().all(|n| (*n % 2) ==0 ));

    let v = vec![6,2,1,9,3,8,11];
    let f1 = v.par_iter().find_any(|&&x| x==9);
    let f2 = v.par_iter().find_any(|&&x| x >8);
    info!("{:?}", f2);
    info!("{:?}", f1);

    let mut vec = vec![String::new(); 100_000];
    vec.par_iter_mut().for_each(|p| {
        let mut rng = rand::rng();
        *p = (0..5).map(|_| rng.sample(&Alphanumeric) as char).collect()
    });

    vec.par_sort_unstable();
    info!("{:?}", vec);

}

struct Person2 {
    age: u32,
}

#[test]
fn it_reduce_test() {
    init();
    let v: Vec<Person2> = vec![
        Person2 { age: 23},
        Person2 { age: 19},
        Person2 { age: 42},
        Person2 { age: 17},
        Person2 { age: 17},
        Person2 { age: 31},
        Person2 { age: 30},
    ];

    let num_over_30 = v.par_iter().filter(|&x| x.age > 30).count() as f32;
    info!("num_over_30: {}", num_over_30);
    let sum_over_30 = v.par_iter()
        .map(|x| x.age)
        .filter(|&x| x > 30)
        .reduce(|| 0, |x,y| x + y);
    info!("sum over 30: {}", sum_over_30);

    let alt_sum_30: u32 = v.par_iter()
        .map(|x| x.age)
        .filter(|&x| x > 30)
        .sum();
    info!("alt sum 30: {}", alt_sum_30);

    let avg_over_30 = sum_over_30 as f32 / num_over_30;
    let alt_avg_over_30 = alt_sum_30 as f32 / num_over_30;
    info!("{}, {}", avg_over_30, alt_avg_over_30);

}

#[test]
fn it_glob_test01() -> anyhow::Result<()> {
    init();
    let options: MatchOptions = Default::default();

    let files: Vec<_> = glob_with("C:\\Users\\user\\Pictures\\*.jpg", options)?
        .filter_map(|x| x.ok())
        .collect();

    if files.len() == 0 {
        info!("No .jpg files found in current directory!");
        return Ok(());
    }

    let thumb_dir = "thumbnails";
    create_dir_all(thumb_dir)?;
    info!("Saving {} thumbnails into '{}'...", files.len(), thumb_dir);

    let image_failures: Vec<_> = files
        .par_iter()
        .map(|path| {
            make_thumbnail(path, thumb_dir, 300).unwrap() 
        })
        .collect();

    Ok(())
}

use error_chain::error_chain;

fn make_thumbnail<PA, PB>(original: PA, thumb_dir: PB, longest_edge: u32) -> anyhow::Result<()> 
    where 
        PA: AsRef<Path>,
        PB: AsRef<Path>
{
    let img = image::open(original.as_ref())?;
    let file_path = thumb_dir.as_ref().join(original);

    Ok(img.resize(longest_edge, longest_edge, image::imageops::FilterType::Nearest)
        .save(file_path)?)
}

fn sha256_diget<R: Read>(mut reader: R) -> anyhow::Result<Digest> {
    let mut context = Context::new(&SHA256);
    let mut buf = [0; 1024];

    loop {
        let count = reader.read(&mut buf)?;
        if count == 0 {
            break;
        }
        context.update(&buf[..count]);
    }
    Ok(context.finish())
}

#[test]
fn it_file_sha256_diget_test01() -> anyhow::Result<()> {
    init();
    let path = "files/file.txt";
    let mut output = File::create(path)?;
    write!(output, "We will generate a digest of this text")?;

    let input = File::open(path)?;
    let reader = BufReader::new(input);
    let digest = sha256_diget(reader)?;

    info!("SHA-256 digest is {}", HEXUPPER.encode(digest.as_ref()));

    Ok(())
}

// 使用 HMAC 摘要对消息进行签名和验证
#[test]
fn it_msg_hmac_sign_test01() -> anyhow::Result<(), Unspecified>{
    init();
    let mut key_value = [0u8; 48];
    let rng = ring::rand::SystemRandom::new();
    rng.fill(&mut key_value)?;

    let key = hmac::Key::new(hmac::HMAC_SHA256, &key_value);

    let msg = "Legitimate and important message.";
    info!("{}", msg);

    let signature = hmac::sign(&key, msg.as_bytes());
    info!("{:?}", &signature);

    hmac::verify(&key, msg.as_bytes(), signature.as_ref())?;

    Ok(())
}


#[test]
fn it_pwd_salt_test01() -> anyhow::Result<(), Unspecified> { 
    init();
    const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
    let n_iter = NonZeroU32::new(100_0000).unwrap();
    let rng = ring::rand::SystemRandom::new();

    let mut salt = [0u8; CREDENTIAL_LEN];
    rng.fill(&mut salt)?;

    let password = "Guess Me If You Can";
    let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(pbkdf2::PBKDF2_HMAC_SHA512,
         n_iter, 
         &salt, 
         password.as_bytes(), 
         &mut pbkdf2_hash);
        
    info!("Salt: {:?}", &salt);
    info!("Salt: {}", HEXUPPER.encode(&salt));

    info!("PBKDF2 hash: {:?}", &pbkdf2_hash);
    info!("PBKDF2 hash: {}", HEXUPPER.encode(&pbkdf2_hash));

    let should_succeed = pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA512,
        n_iter,
        &salt,
        password.as_bytes(),
        &pbkdf2_hash,
    );

    let wrong_password = "Definitely not the corret password";
    let should_fail = pbkdf2::verify(pbkdf2::PBKDF2_HMAC_SHA512, 
        n_iter, 
        &salt, 
        wrong_password.as_bytes(), 
        &pbkdf2_hash);

    info!("{:?}", should_succeed.is_ok());
    info!("{:?}", should_fail.is_ok());
    Ok(())
}

use bitflags::bitflags ;

bitflags! {
    struct MyFlags: u32 {
        const FLAG_A       = 0b00000001;
        const FLAG_B       = 0b00000010;
        const FLAG_C       = 0b00000100;
        const FLAG_ABC     = Self::FLAG_A.bits()
                           | Self::FLAG_B.bits()
                           | Self::FLAG_C.bits();
    }
}

impl MyFlags {
    pub fn clear(&mut self) -> &mut MyFlags {
        // self.;
        self
    }
}

