#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
#![allow(dead_code)]
#![allow(unused_variables)]

use core::num;
use std::{collections::HashMap, hash::Hash, sync::{Arc, Mutex}};

use bytes::Bytes;
use log::info;
use mini_redis::{Connection, Frame};
use tokio::net::TcpStream;

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

type ShardedDb = Arc<Vec<Mutex<HashMap<String, Vec<u8>>>>>;

fn new_shared_db(num_shareds: usize) -> ShardedDb {
    let mut db = Vec::with_capacity(num_shareds);
    for _ in 0..num_shareds {
        db.push(Mutex::new(HashMap::new()));
    }
    return Arc::new(db);
}


pub async fn process(socket: TcpStream, db: Db) {

    // 使用 hashmap 来存储 redis 的数据
    // let mut db = HashMap::new();

    // mini-reds 提供的便利函数，使用返回的 connection 可以用于从 socket
    // 中读取数据并解析为数据帧
    let mut connection = Connection::new(socket);
    
    // 使用 read_frame 方法从连接获取一个数据帧:  一个 redis 命令 + 相应的数据 
    while let Some(frame) = connection.read_frame().await.unwrap() {
        info!("Got: {:?}", frame);
        let response = match mini_redis::Command::from_frame(frame).unwrap() {
            mini_redis::Command::Set(cmd) => {
                // 值被存储为 Vec<u8> 的形式
                let mut db = db.lock().unwrap();
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            },
            mini_redis::Command::Get(cmd) => {
                let db = db.lock().unwrap();
                if let Some(value) = db.get(cmd.key()) {
                    // Frame::Bulk 期待数据的类型是 Bytes， 该类型会在后面章节讲解,
                    // 些时，你只要知道 &Vec<u8> 可以使用 into() 方法转换成 Bytes 类型
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        // 将请求响应返回给客户端
        connection.write_frame(&response).await.unwrap();
    }
}


#[derive(Debug)]
enum Command {
    Get {
        key: String,
    },
    Set {
        key: String,
        val: Bytes,
    }
}