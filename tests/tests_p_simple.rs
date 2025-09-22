
use std::{sync::{mpsc, Arc, Mutex}, thread, time::Duration};
use log::info;
use vehicle_manager_axum::{flexible_test, init};

#[test]
fn itest_01() {
    init();
    info!("test2233");
    
    let handle = thread::spawn(|| {
        for i in 1..10 {
            info!("hi number {} from the spawned thread!", i);
            thread::sleep(Duration::from_millis(1));
        }
    });

    handle.join().unwrap();

    for i in 1..5 {
        info!("Hi nubmer {} from the main thread!", i);
        thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn it_thread_move_test() {
    init();

    let v = vec![1,2,3];
    let handle = thread::spawn(move || {
        info!("Here's a vector: {:?}", v);
    });

    handle.join().unwrap();
}

#[test]
fn it_channel_test() {
    init();

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let val = String::from("hi");
        tx.send(val).unwrap();
    });

    let received = rx.recv().unwrap();
    info!("Got: {}", received);
}

// 发送多值， 看到接收者在等待
#[test]
fn it_ms_channel_test() {
    init();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let vals = vec![
            String::from("hi"),
            String::from("from"),
            String::from("the"),
            String::from("thread"),
        ];

        for val in vals {
            tx.send(val).unwrap();
            thread::sleep(Duration::from_millis(1));
        }
    });

    for received in rx {
        info!("Got: {}", received);
    }
}

// 通过克隆创建多个发送者
#[test]
fn it_channel_test02(){ 
    init();
    let (tx, rx) = mpsc::channel();
    let tx1 = mpsc::Sender::clone(&tx);
    thread::spawn(move || {
        let vals = vec![
            String::from("1: hi"),
            String::from("1: from"),
            String::from("1: the"),
            String::from("1: thread"),
        ];

        for val in vals {
            tx1.send(val).unwrap();
            thread::sleep(Duration::from_millis(1));
        }
    });

    thread::spawn(move || {
        let vals = vec![
            String::from("hi"),
            String::from("from"),
            String::from("the"),
            String::from("thread"),
        ];

        for val in vals {
            tx.send(val).unwrap();
            thread::sleep(Duration::from_millis(1));
        }
    });

    for received in rx {
        info!("Got: {}", received);
    }
}   

#[test]
fn it_mutex_test01() {
    init();
    let m = Mutex::new(5);
    {
        let mut num = m.lock().unwrap();
        *num = 6;
    }
    info!("m = {:?}", m);
}

// 使用 Arc<T> 来进行原子引用计数
#[test]
fn it_arc_mutex_test01() {
    init();
    info!("Test");
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    info!("Result: {}", *counter.lock().unwrap());
}

#[test]
fn it_flexible_test() {
    init();
    let string = String::from("example");
    let result1 = flexible_test::frobinicate3::<&str>(string.as_ref());
    info!("Result1: {:?}", result1);
}