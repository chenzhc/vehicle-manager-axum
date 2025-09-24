#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::{collections::HashMap, rc::Rc, sync::{mpsc, Arc, Mutex}, thread};

use async_std::task::yield_now;
use bytes::Bytes;
use log::info;
use mini_redis::{client, Result};
use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpSocket, TcpStream}, task};
use vehicle_manager_axum::{flexible_test::say_to_world, init, redis_test::process};

#[tokio::test]
async fn it_redis_client_test() -> Result<()> {
    init();
    let mut client = client::connect("localhost:6378").await?;

    client.set("hello", "world".into()).await?;

    let result = client.get("hello").await?;

    info!("从服务器端获取到结果 = {:?}", result);
    Ok(())
}

#[tokio::test]
async fn it_say_to_world_test() {
    init();
    let op = say_to_world();

    info!("hello");

    info!("{}", op.await);
}


#[tokio::test]
async fn it_redis_server_test() {
    init();

    let listener = TcpListener::bind("0.0.0.0:6378").await.unwrap();
    info!("Listening");

    let db = Arc::new(Mutex::new(HashMap::new()));
    
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();

        info!("Accepted");
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

#[tokio::test]
async fn it_test_spwan_test() {
    init();
    let handle = tokio::spawn(async {
        10086
    });

    let out = handle.await.unwrap();
    info!("Got: {}", out);
}

#[tokio::test]
async fn it_vec_test() {
    init();
    let v = vec![1,2,3];

    let _ = task::spawn(async move {
        info!("Here's a vec: {:?}", v);
    }).await;

    let _ = task::spawn(async {
        {
            let rc = Rc::new("hello");
            info!("{}", rc);
        }
        yield_now().await;
    }).await;

}

#[tokio::test]
async fn it_tokio_channel_test() {
    init();
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    let tx2 = tx.clone();

    tokio::spawn(async move {
        let _ = tx.send("sending from first handle").await;
    });

    tokio::spawn(async move {
        let _ = tx2.send("sending from second handle").await;
    });

    while let Some(message) = rx.recv().await {
        info!("Got = {}", message);
    }
}



#[tokio::test]
async fn it_tcp_copy_test() -> anyhow::Result<()> {
    init();
    let socket = TcpStream::connect("127.0.0.1:6142").await?;
    let (mut rd, mut wr) = tokio::io::split(socket);

    tokio::spawn(async move {
        wr.write_all(b"hello\r\n").await?;
        wr.write_all(b"world\r\n").await?;

        Ok::<_, io::Error>(())
    });

    let mut buf = vec![0; 128];

    loop {
        let n = rd.read(&mut buf).await?;

        if n == 0 {
            break;
        }
        info!("Got {:?}", &buf[..n]);
    }

    Ok(())
}

#[test]
fn it_crossbem_thread_test01() {
    init();

    crossbeam_utils::thread::scope(|s| {
        for i in 0..5 {
            s.spawn(move |_| {
                info!("Thread {i} is running");
            });
        }
    }).unwrap();
    info!("All threads have fnished.");

}

#[tokio::test]
async fn it_channel_test03() {
    init();
    let (tx, rx) = crossbeam_channel::unbounded();

    thread::spawn(move || {
        for i in 0..5 {
            tx.send(i).unwrap();
            info!("Sent: {}", i);
        }
    });

    for recived in rx {
        info!("Received: {}", recived);
    }
}

#[test]
fn it_queue_test() {
    init();
    let queue = crossbeam_queue::SegQueue::new();

    queue.push(1);
    queue.push(2);

    while let Some(value) = queue.pop() {
        info!("Popped: {}", value);
    }

}

#[test]
fn it_epoch_test01() {
    init();
    let collector = crossbeam_epoch::Collector::new();
    let handle = collector.register();
    let atomic = crossbeam_epoch::Atomic::null();
 
}