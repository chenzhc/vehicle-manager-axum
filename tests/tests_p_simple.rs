
#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
#![allow(dead_code)]
#![allow(unused_variables)]
use std::{sync::{mpsc, Arc, Mutex}, thread, time::{Duration, Instant}};
use log::info;
use vehicle_manager_axum::{flexible_test::{self, get_memory_location, get_str_at_location, read_from_file1, read_from_file1_async, read_from_file2, read_from_file2_async, split_at_mut, AsyncTimer, HelloMacro, ReadFileFutre, Sunface, Sunfei}, init, vecust};

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

// 同步例子
#[test]
fn it_read_from_file_test01() {
    init();
    info!("Hello before reading file");
    let file_contents = read_from_file1();
    info!("{}", file_contents);
    info!("Hello after reading file1");
    let file2_contnet = read_from_file2();
    info!("{}", file2_contnet);
    info!("Hello after reading file2");
}

// 多线程例子
#[test]
fn it_read_from_file_thread_test() {
    init();
    info!("Hello before reading file!");
    let handle1 = thread::spawn(move || {
        let file1_content = read_from_file1();
        info!("{}", file1_content);
    });
    let handle2 = thread::spawn(move || {
        let file2_content = read_from_file2();
        info!("{}", file2_content);
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}

// 异步例子
#[tokio::test]
async fn it_read_from_file_async_test() {
    init();
    info!("Hello before reading file!");

    let h1 = tokio::spawn(async {
        let _file1_content = read_from_file1_async().await;
        info!("{}", _file1_content);
    });

    let h2 = tokio::spawn(async {
        let file2_content = read_from_file2_async().await;
        info!("{:?}", file2_content);
    });

    let _ = tokio::join!(h1, h2);
}

#[tokio::test]
async fn it_future_test() {
    init();

    info!("Hello before reading file");
    let h1 = tokio::spawn(async {
        let future1 = ReadFileFutre { };
        future1.await;
    });

    let _ = tokio::join!(h1);
}

#[tokio::test]
async fn it_future_test02() {
    init();
    let h1 = tokio::spawn(async {
        let future1 = AsyncTimer {
            expiration_time: Instant::now() + Duration::from_millis(4000),
        };
        info!("{:?}", future1.await);
    });

    let _ = tokio::join!(h1);
}

#[test]
fn it_unsafe_test01() {
    init();
    let mut num = 5;
    let r1 = &num as *const i32;
    unsafe {
        info!("r1 is: {}", *r1);
    }
}

#[test]
fn it_read_ptr_test() {
    init();
    let (pointer, length) = get_memory_location();
    let message = get_str_at_location(pointer, length);
    info!("The {} bytes at 0x{:X} stored: {}", length, pointer, message);
}

#[test]
fn it_unsafe_pointer_test02() {
    init();
    let a = 1;
    let b: *const i32 = &a as *const i32;
    let c: *const i32 = &a;
    unsafe {
        info!("{}", *b);
    }

    let a: Box<i32> = Box::new(10);
    let b: *const i32 = &*a;
    let c: *const i32 = Box::into_raw(a);
    unsafe {
        info!("{}", *b);
        info!("{}", *c);
    }


}

#[test]
fn it_unsafe_fn_test() {
    init();
    unsafe {
        flexible_test::dangerous();
    }
    info!("Test");
}

#[test]
fn it_unsafe_slice_test01() {
    init();
    let mut v = vec![1,2,3,4,5,6];
    let r = &mut v[..];
    let (a, b) = split_at_mut(r, 3);

    info!("{:?}", a);
    info!("{:?}", b);
}

#[test]
fn it_ffi_abs_test01() {
    init();
    unsafe {
        info!("Absolute value of -3 according to C: {}", flexible_test::abs(-3));
    }
}

#[test]
fn it_mac_vec_test01() {
    init();
    let v = vecust![1, 2, 3];
    info!("{:?}", v);
}

#[test]
fn it_hello_macro_test() {
    init();
    Sunfei::hello_macro();
    Sunface::hello_macro();

}