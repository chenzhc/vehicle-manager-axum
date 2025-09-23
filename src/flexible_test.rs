use std::{fs, io::{BufRead, BufReader, Read, Write}, marker::PhantomPinned, net::TcpStream, pin::Pin, slice::from_raw_parts, str::from_utf8_unchecked, sync::{mpsc, Arc, Mutex}, task::Poll, thread::{self, JoinHandle}, time::{Duration, Instant}};
use async_std::io::{ReadExt, WriteExt};
use hello_macro_derive::HelloMacro;
use log::info;
use log4rs::append::file;


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


pub async fn do_something() {
    info!("go go go!");
}


pub async fn hello_world() {
    let _  = hello_cat().await;
    info!("hello, world!");
}

pub async fn hello_cat() {
    info!("hello, kitty!");
}

pub struct Song {
    author: String,
    name: String,
}

pub async fn learn_song() -> Song {
    Song {
        author: "test".to_string(),
        name: String::from("juhuaTai").to_string(),
    }
}

pub async fn sing_song(song: Song) {
    info!("This is a song author: {}, name: {}, sing.. {}",
        song.author, song.name, "test..."
    );
}

pub async fn dance() {
    info!("dancing...");
}

pub async fn learn_and_sing() {
    let song = learn_song().await;

    sing_song(song).await;

}

pub async fn async_main() {
    let f1 = learn_and_sing();
    let f2 = dance();

    futures::join!(f1, f2);
}


#[derive(Debug)]
pub struct TestSelRef {
    pub a: String,
    pub b: *const String,
    pub _marker: PhantomPinned,
}

impl TestSelRef {
    pub fn new(txt: &str) -> Self {
        TestSelRef { 
            a: String::from(txt), 
            b: std::ptr::null(), 
            _marker: PhantomPinned,
        }
    }

    pub fn init(self: Pin<&mut Self>) {
        let self_ptr: *const String = &self.a;
        let this = unsafe {
            self.get_unchecked_mut()
        };
        this.b = self_ptr;
    }

    pub fn a(self: Pin<&Self>) -> &str {
        &self.get_ref().a
    }

    pub fn b(self: Pin<&Self>) -> &String {
        assert!(!self.b.is_null(), "TestSelRef::b called without TestSelRef::init being called first");
        unsafe {
            &*(self.b)
        }
    }
}

#[derive(Debug)]
pub struct TestBox {
    pub a: String,
    pub b: *const String,
    pub _marker: PhantomPinned,
}

impl TestBox {
    pub fn new(txt: &str) -> Pin<Box<Self>> {
        let t = TestBox {
            a: String::from(txt),
            b: std::ptr::null(),
            _marker: PhantomPinned,
        };
        let mut boxed = Box::pin(t);
        let self_ptr: *const String = &boxed.as_ref().a;
        unsafe {
            boxed.as_mut()
                .get_unchecked_mut()
                .b = self_ptr;
        }
        boxed
    }

    pub fn a(self: Pin<&Self>) -> &str {
        &self.get_ref().a
    }

    pub fn b(self: Pin<&Self>) -> &String {
        unsafe {
            &*(self.b)
        }
    }
}

pub async fn blocks() {
    let my_string = "foo".to_string();

    let future_one = async move {
        info!("{}", my_string);
    };

    let _ = futures::join!(future_one);
}


pub async fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        info!("sleep...5");        
        async_std::task::sleep(Duration::from_secs(5)).await;
        ("HTTP/1.1 200 Ok\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };
    let contents = fs::read_to_string(filename).unwrap();

    let response = format!("{status_line} {contents}");
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

pub async fn handle_connection_async(mut stream: async_std::net::TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await.unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        info!("sleep...5");        
        async_std::task::sleep(Duration::from_secs(5)).await;
        ("HTTP/1.1 200 Ok\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };
    let contents = fs::read_to_string(filename).unwrap();

    let response = format!("{status_line} {contents}");
    stream.write(response.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();
}

pub fn handle_connection3(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1"  => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    let response = format!("{}\r\nContent-Length: {}\r\n\r\n{}", status_line, length, contents);
    stream.write_all(response.as_bytes()).unwrap();
    // info!("Request: {:?}", http_request);

}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers: workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)  
        where 
            F: FnOnce() + Send + 'static, 
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            info!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct  Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // let job = receiver.lock().unwrap().recv().unwrap();
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    info!("Worker {} got a job; executing...", id);
                    job();
                },
                Err(_) => {
                    info!("Worker {} disconnected; shutting down.", id);
                    break;
                }
            }
        });

        Worker { 
            id: id, 
            thread: Some(thread)
        }
    }
}

// struct Job;


pub async fn say_to_world() -> String {
    return String::from("world");
}

