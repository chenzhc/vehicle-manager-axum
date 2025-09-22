use std::{slice::from_raw_parts, str::from_utf8_unchecked, task::Poll, thread, time::{Duration, Instant}};

use hello_macro_derive::HelloMacro;
use log::info;


pub fn frobinicate3<T: AsRef<str>>(s: T) -> T {
    s
}


pub fn read_from_file1() -> String {
    thread::sleep(Duration::new(4,0));
    String::from("Hello, there from file 1")
}

pub fn read_from_file2() -> String {
    thread::sleep(Duration::new(2, 0));
    String::from("Hello, there from file 2")
}

pub async fn read_from_file1_async() -> String {
    thread::sleep(Duration::new(4, 0));
    info!("{:?}", "Processing file 1");
    String::from("Hello, there from file 1")
}

pub async fn read_from_file2_async() -> String {
    thread::sleep(Duration::new(2, 0));
    info!("{:?}", "Processing file 2");
    String::from("Hello, there from file 2")
}

pub struct ReadFileFutre {

}

impl Future for ReadFileFutre {
    type Output = String;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        info!("Tokio! Stop polling me");
        cx.waker().wake_by_ref();
        Poll::Ready(String::from("Hello, there from file 1"))
    }
}

pub struct AsyncTimer {
    pub expiration_time: Instant,
}

impl Future for AsyncTimer {
    type Output = String;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.expiration_time {
            info!("Hello, it's time for Future 1");
            Poll::Ready(String::from("Future 1 has completed"))
        } else {
            info!("Hello, it's not yet time for Future 1, Going to sleep");
            let waker = cx.waker().clone();
            let expiration_time = self.expiration_time;
            thread::spawn(move || {
                let current_time = Instant::now();
                if current_time < expiration_time {
                    thread::sleep(expiration_time - current_time);
                }
                waker.wake();
            });
            Poll::Pending
        }
    }
}

// 获取字符串的内存地址和长度
pub fn get_memory_location() -> (usize, usize) {
    let string = "Hello world!";
    let pointer = string.as_ptr() as usize;
    let length = string.len();
    return (pointer, length);
}

// 在指定的内存地址读取字符串
pub fn get_str_at_location(pointer: usize, length: usize) -> &'static str {
    unsafe {
        return from_utf8_unchecked(from_raw_parts(pointer as *const u8, length));
    }
}

pub unsafe fn dangerous() {
    info!("dangerous");
}

pub fn split_at_mut(slice: &mut [i32], mid: usize) -> (&mut [i32], &mut [i32]) {
    let len = slice.len();
    let ptr = slice.as_mut_ptr();

    assert!(mid <= len);
    unsafe {
        return (
            std::slice::from_raw_parts_mut(ptr, mid),
            std::slice::from_raw_parts_mut(ptr.add(mid), len - mid)
        );
    }
}

unsafe extern "C" {
    pub unsafe fn abs(input: i32) -> i32;
}

#[macro_export]
macro_rules! vecust {
    ( $( $x:expr ), * ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}

pub trait HelloMacro {
    fn hello_macro();    
}


#[derive(HelloMacro)]
pub struct Sunfei;

#[derive(HelloMacro)]
pub struct Sunface;