#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
#![allow(dead_code)]
#![allow(unused_variables)]
use core::borrow;
use std::{cell::{RefCell, UnsafeCell}, collections::HashMap, marker::PhantomPinned, pin::Pin, ptr::NonNull, rc::{Rc, Weak}, sync::{atomic::{AtomicU64, AtomicUsize, Ordering}, LazyLock, Mutex, Once, OnceLock}, thread::{self, JoinHandle}};
use lazy_static::lazy_static;
use log::info;

// 主人
struct Owner {
    name: String,
    gadgets: RefCell<Vec<Weak<Gadget>>>,
}

// 工具
struct Gadget {
    id: i32,
    owner: Rc<Owner>,
}

pub trait Message {
    fn send(&self, msg: String);    
}

struct MsgQueue {
    msg_cache: RefCell<Vec<String>>,
}

impl Message for MsgQueue {
    fn send(&self, msg: String) {
        self.msg_cache.borrow_mut().push(msg);
    }
}

#[derive(Debug)]
struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

#[derive(Debug)]
struct SelfRef {
    value: String,
    pointer_to_value: *mut String,
}

impl SelfRef {
    fn new(text: &str) -> Self {
        SelfRef { 
            value: String::from(text), 
            pointer_to_value: std::ptr::null_mut(),
        }
    }

    fn init(&mut self) {
        let self_ref: *mut String = &mut self.value;
        self.pointer_to_value = self_ref;
    }

    fn value(&self) -> &str {
        &self.value
    }

    fn pointer_to_value(&self) -> &String {
        unsafe {
            &*(self.pointer_to_value)
        }
    }
}

// 下面是一个自引用数据结构，因为 slice字段是一个指针，指向了data字段
// 我们无法使用普通引用来实现， 因为违背了Rust的编译规则
// 因此，这里我们使用了一个裸指针，通过NonNull来确保它不会为null 
struct Unmoveable {
    data: String,
    slice: NonNull<String>,
    _pin: PhantomPinned,
}

impl Unmoveable {
    // 为了确保函数返回时数据的所有权不会被转移，我们将它放在堆上，唯一的访问方式就是通过指针
    fn new(data:String) -> Pin<Box<Self>> {
        let res = Unmoveable {
            data,
            // 只有在数据到位时，才创建指针，否则数据会在开始之前就被转换所有权
            slice: NonNull::dangling(),
            _pin: PhantomPinned,
        };
        let mut boxed = Box::pin(res);

        let slice = NonNull::from(&boxed.data);
        // 这里其实安全的，因为修改一个字段不会转移整个结构体的所有权
        unsafe  {
            let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).slice = slice;
        }
        return boxed;
    }
}

thread_local! {
    static FOO: RefCell<i32> = RefCell::new(1);
}


static mut VAL: usize = 0;
static INIT: Once = Once::new();

enum Fruit {
    Apple(u8),
    Orange(String),
}

lazy_static! {
    static ref MUTEX1: Mutex<i64> = Mutex::new(0);
    static ref MUTEX2: Mutex<i64> = Mutex::new(0);
}

const N_TIMES: u64 = 10_000_000;
const N_THREADS: usize = 10;

static R: AtomicU64 = AtomicU64::new(0);

fn add_n_times(n: u64) -> JoinHandle<()> {
    thread::spawn(move || {
        for _ in 0..n {
            R.fetch_add(1, Ordering::Relaxed);
        }
    })
}

#[derive(Debug)]
struct MyBox(*const u8);
unsafe impl Send for MyBox {

}
unsafe impl Sync for MyBox {

}

const MAX_ID: usize = usize::MAX / 2;
static REQUEST_RECV: AtomicUsize  = AtomicUsize::new(0);

struct Factory {
    factory_id: usize,
}

static GLOBAL_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn generate_id() -> usize {
    let current_val = GLOBAL_ID_COUNTER.load(Ordering::Relaxed);
    if current_val > MAX_ID {
        panic!("Factory ids overflowed");
    }
    GLOBAL_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let next_id = GLOBAL_ID_COUNTER.load(Ordering::Relaxed);
    if next_id > MAX_ID {
        panic!("Factory ids overflowed");
    }
    return next_id;
}

impl Factory {
    fn new() -> Self {
        Self {
            factory_id: generate_id()
        }
    }
}

lazy_static! {
    static ref NAMES: Mutex<String> = Mutex::new(String::from("Sunface, Jack, Allen"));
    static ref HASHMAP: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(0, "foo");
        m.insert(1, "bar");
        m.insert(2, "baz");
        m
    };
}

#[derive(Debug)]
struct Config {
    a: String,
    b: String,
}
static mut CONFIG: Option<&mut Config> = None;

fn init() -> Option<&'static mut Config> {
    let c = Box::new(Config {
        a: "A".to_string(), 
        b: "B".to_string(), 
    });

    Some(Box::leak(c))
}

#[derive(Debug)]
struct Logger;

static LOGGER: OnceLock<Logger> = OnceLock::new();
static LOGGER2: LazyLock<Logger> = LazyLock::new(Logger::new);

impl Logger {
    fn new() -> Logger {
        info!("Logger is being created...");
        Logger
    }

    fn global() -> &'static Logger {
        // 获取或初始化 Logger 
        LOGGER.get_or_init(|| {
            // 初始化打印
            info!("Logger is being created...");
            Logger
        })
    }

    fn log(&self, message: String) {
        info!("{}", message);
    }
}



#[cfg(test)]
mod tests {
    use core::num;
    use std::{cell::{Cell, RefCell}, f32::consts::LOG10_2, hint, rc::Rc, sync::{atomic::AtomicUsize, mpsc::{self, Receiver, Sender}, Arc, Barrier, Condvar, Mutex, RwLock}, thread, time::{Duration, Instant}};
    use log::info;
    use smol::fs::windows;
    use tokio::sync::Semaphore;
    use super::*;

    #[test]
    fn it_result_test01() {
        crate::init();
        let s1 = Some("some1");
        let s2 = Some("some2");
        let n: Option<&str> = None;

        let o1: Result<&'static str, &str> = Ok("ok1");
        let o2: Result<&str, &str> = Ok("ok2");
        let e1: Result<&str, &str> = Err("error1");
        let e2: Result<&str, &str> = Err("error2");

        info!("{:?}", s1.or(s2));
        info!("{:?}", s1.or(n));
        info!("{:?}", n.or(s1));
        info!("{:?}", n.or(n));


    }

    #[test]
    fn it_lazylock_test01() {
        crate::init();
        // 子线程中调用
        let handle = thread::spawn(|| {
            let logger = &LOGGER2;
            logger.log("thread message".to_string());
        });

        // 主线程调用
        let logger = &LOGGER2;
        logger.log("some message".to_string());

        let logger2 = &LOGGER2;
        logger2.log("other message".to_string());

        handle.join().unwrap();
    }

    #[test]
    fn it_logger_test01() {
        crate::init();
        // 子线程中调用
        let handle = thread::spawn(|| {
            let logger = Logger::global();
            logger.log("thread message".to_string());

        });

        // 主线程调用
        let logger = Logger::global();
        logger.log("some message".to_string());

        let logger2 = Logger::global();
        logger2.log("other message".to_string());
        handle.join().unwrap();
    }

    #[test]
    fn it_config_test01() {
        unsafe {
            CONFIG = init();
        }
    }

    #[test]
    fn it_static_test02() {
        crate::init();
        for _ in 0..10 {
            REQUEST_RECV.fetch_add(1, Ordering::Relaxed);
        }
        info!("当前用户请求数 {:?}", REQUEST_RECV.load(Ordering::Relaxed));

        let factory = Factory::new();
        info!("{:?}", factory.factory_id);

        let mut v = NAMES.lock().unwrap();
        v.push_str(", Myth");
        info!("{}", v);

        info!("The entry for '0' is {}", HASHMAP.get(&0).unwrap());
        info!("The entry for 1 is {}", HASHMAP.get(&1).unwrap());

    }

    #[test]
    fn it_const_test01() {
        crate::init();
        info!("用户ID允许的最大值是 {}", MAX_ID);

    }

    #[test]
    fn it_atomic_test03() {
        crate::init();
        let b = &MyBox(5 as *const u8);
        let v = Arc::new(Mutex::new(b));
        let t = thread::spawn(move || {
            let v1 = v.lock().unwrap();
            info!("{:?}", v1);
        });

        t.join().unwrap();
    }

    #[test]
    fn it_mybox_test01() {
        crate::init();
        let p = MyBox(5 as *const u8);
        let t = thread::spawn(move || {
            info!("{:?}", p);
        });
        t.join().unwrap();
    }

    #[test]
    fn it_atomic_test02() {
        crate::init();
        let spinlock = Arc::new(AtomicUsize::new(1));

        let spinlock_clone = Arc::clone(&spinlock);
        let thread = thread::spawn(move || {
            spinlock_clone.store(0, Ordering::SeqCst);
            info!("thread...");
        });

        // 等待其它线程释放锁
        while spinlock.load(Ordering::SeqCst) != 0 {
            info!("whil....");
            hint::spin_loop();
        }

        if let Err(panic) = thread.join() {
            info!("Thread had an error: {:?}", panic);
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn it_atomic_test01() {
        crate::init();
        let s = Instant::now();
        let mut threads = Vec::with_capacity(N_THREADS);

        for _ in 0..N_THREADS {
            threads.push(add_n_times(N_TIMES));
        }

        for thread in threads {
            thread.join().unwrap();
        }

        info!("{}", R.load(Ordering::Relaxed));
        info!("{:?}", Instant::now().duration_since(s));

    }

    #[tokio::test]
    async fn it_semaphore_test() {
        crate::init();
        let semaphore = Arc::new(Semaphore::new(3));
        let mut join_handles = Vec::new();

        for _ in 0..5 {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            join_handles.push(tokio::spawn(async move {

                info!("thread...");
                drop(permit);
            }));
        }

        for handle in join_handles {
            handle.await.unwrap();
        }
    }

    #[test]
    fn it_cond_test01() {
        crate::init();
        let flag = Arc::new(Mutex::new(false));
        let cond = Arc::new(Condvar::new());
        let cflag = flag.clone();
        let ccond = cond.clone();

        let hdl = thread::spawn(move || {
            let mut lock = cflag.lock().unwrap();
            let mut counter = 0;

            while counter < 3 {
                while !*lock {
                    // wait 方法会接收一个 MutexGuard<'a, T>, 
                    // 且它会自动地暂时释放这个锁， 使其他线程可
                    // 以拿到锁并进行数据更新.
                    // 同时当前线程在此处会被阻塞，直到被其他地方 notify后，
                    // 它会将原本的 MutexGuard<'a, T> 还给我们，即重新获取
                    // 到了锁，同时唤醒了此线程.
                    lock = ccond.wait(lock).unwrap();
                }

                *lock = false;

                counter += 1;
                info!("inner counter: {}", counter);
            }
        });

        let mut counter = 0;
        loop {
            thread::sleep(Duration::from_millis(1000));
            *flag.lock().unwrap() = true;
            counter += 1;
            if counter > 3 {
                break;
            }
            info!("outside counter: {}", counter);
            cond.notify_one();
        }

        hdl.join().unwrap();
        info!("{:?}", flag);
    }

    #[test]
    fn it_rwlock_test() {
        crate::init();

        let lock = RwLock::new(5);

        // 同一时间允许多个读
        {
            let r1 = lock.read().unwrap();
            let r2 = lock.read().unwrap();
            info!("{}", *r1);
            info!("{}", *r2);
        } // 读锁在些外被 drop 

        // 同一时间只允许一个写
        {
            let mut w = lock.write().unwrap();
            *w += 1;
            info!("{}", *w);

            // 以下代码会阻塞发生死锁， 因为读和写不允许同时存在
            // 写锁直到该语句块结束才被释放，因此下面的读锁依然处理 w 的作用域中
            // let r1 = lock.read();
            // info!("{:?}", r1);
        } // 写锁在此处被 drop 
    }

    #[test]
    fn it_dead_lock_test2() {
        crate::init();
        // 存放子线程的句柄
        let mut children = vec![];
        for i_thread in 0..2 {
            let cd1 = thread::spawn(move || {
                for _ in 0..1 {
                    // 线程
                    if i_thread % 2 == 0 {
                        // 锁住 MUTEX1 
                        let guard = MUTEX1.lock().unwrap();
                        info!("线程 {} 锁住了 MUTEX1, 接着准备去锁 MUTEX2!", i_thread);

                        // 当前线程睡眠一小会儿， 等待线程2锁住 MUTEX2
                        thread::sleep(Duration::from_millis(10));

                        // 去锁 MUTEX2
                        let guard = MUTEX2.try_lock();
                        info!("线程 {} 获取 MUTEX2 锁的结果: {:?}", i_thread, guard);
                    } else {
                        // 线程2 
                        // 锁住 MUTEX2
                        let _guard = MUTEX2.lock().unwrap();
                        info!("线程 {} 锁住了 MUTEX2, 接着准备去锁 MUTEX1!", i_thread);
                        thread::sleep(Duration::from_millis(10));
                        let guard = MUTEX1.try_lock();
                        info!("线程 {} 获取 MUTEX1 锁的结果: {:?}", i_thread, guard);
                    }
                }
            });
            children.push(cd1);
        }

        // 等子线程完成 
        for child in children {
            let _ = child.join();
        }

        info!("死锁没有发生");
    }

    #[test]
    fn it_dead_lock_test() {
        crate::init();
        let data = Mutex::new(0);
        let d1 = data.lock();
        let d2 = data.lock();

    }

    #[test]
    fn it_channel_test10() {
        crate::init();

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
    fn it_channel_test09() {
        crate::init();
        let m = Mutex::new(5);

        let mut num = m.lock().unwrap();
        *num = 6;
        drop(num);

        let mut num1 = m.lock().unwrap();
        *num1 = 7;
        drop(num1);

        info!("m = {}", m.lock().unwrap());

    }

    #[test]
    fn it_channel_test08() {
        crate::init();
        // 使用 Mutex 结构体的关联函数创建新的互斥锁实例
        let m = Mutex::new(5);
        {
            let mut num = m.lock().unwrap();
            *num = 6;
        }
        info!("m = {}", *m.lock().unwrap());

    }

    #[test]
    fn it_channel_test07() {
        crate::init();
        let (tx, rx) = mpsc::channel();
        let num_threads = 3;
        for i in 0..num_threads {
            let thread_send = tx.clone();
            thread::spawn(move || {
                thread_send.send(i).unwrap();
                info!("thread {:?} finished", i);
            });
        }

        drop(tx);

        for x in rx {
            info!("Got: {}", x);
        }
        info!("finished iterating");
    }

    #[test]
    fn it_channel_type_test01() {
        crate::init();
        let (tx, rx): (Sender<Fruit>, Receiver<Fruit>) = mpsc::channel();

        tx.send(Fruit::Orange("sweet".to_string())).unwrap();
        tx.send(Fruit::Apple(2)).unwrap();

        for _ in 0..2 {
            match rx.recv().unwrap() {
                Fruit::Apple(count) => info!("received {} apples", count),
                Fruit::Orange(flavor) => info!("received {} orangle", flavor),
            }
        }
    }

    #[test]
    fn it_channel_test06() {
        crate::init();
        let (tx, rx) = mpsc::sync_channel(0);

        let handle = thread::spawn(move || {
            info!("发关之前");
            tx.send(1).unwrap();
            info!("发送之后");
        });

        info!("睡眠之前");
        thread::sleep(Duration::from_secs(3));
        info!("睡眠之后");

        info!("received: {}", rx.recv().unwrap());
        handle.join().unwrap();

    }

    #[test]
    fn it_channel_test05() {
        crate::init();
        let (tx, rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            info!("发送之前");
            tx.send(1).unwrap();
            info!("发送之后");
        });

        info!("睡眠之前");
        thread::sleep(Duration::from_secs(3));
        info!("睡眠之后");

        info!("received {}", rx.recv().unwrap());
        handle.join().unwrap();

    }

    #[test]
    fn it_channel_test04() {
        crate::init();
        let (tx, rx) = mpsc::channel();
        let tx1 = tx.clone();
        thread::spawn(move || {
            tx.send(String::from("hi from raw tx")).unwrap();
        });

        thread::spawn(move || {
            tx1.send(String::from("hi from cloned tx")).unwrap();
        });

        for received in rx {
            info!("Got: {}", received);
        }
    }

    #[test]
    fn it_channel_test03() {
        crate::init();

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
                thread::sleep(Duration::from_secs(1));
            }
        });

        for received in rx {
            info!("Got: {}", received);
        }
    }

    #[test]
    fn it_channel_test02() {
        crate::init();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let s = String::from("我，飞走咯！");
            tx.send(s).unwrap();
            // info!("val is {}", s);
        });

        let received = rx.recv().unwrap();
        info!("Got: {}", received);

    }

    #[test]
    fn it_channel_test() {
        crate::init();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            tx.send(1).unwrap();
        });

        // info!("receive {}", rx.recv().unwrap());
        info!("receive{:?}", rx.try_recv());
        info!("receive{:?}", rx.try_recv());
        info!("receive{:?}", rx.try_recv());

    }

    #[test]
    fn it_once_init_test() {
        crate::init();
        let handle1 = thread::spawn(move || {
            INIT.call_once(|| {
                unsafe {
                    VAL = 1;
                    info!("init val one");
                }
            });
        });

        let handle2 = thread::spawn(move || {
            INIT.call_once(||{
                unsafe {
                    VAL = 2;
                    info!("init val two");
                }
            });
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        info!("{}", unsafe {
            VAL
        });

    }

    #[test]
    fn it_thread_local_test() {
        crate::init();
        FOO.with(|f| *f.borrow_mut() += 2);

        thread::spawn(|| {
            FOO.with(|f| *f.borrow_mut() += 3);
            FOO.with(|f| info!("Thread: {}", f.borrow()));
        }).join().unwrap();

        FOO.with(|f| info!("Main: {}", f.borrow()));
    }

    #[test]
    fn it_barrier_test() {
        crate::init();
        let mut handles = Vec::with_capacity(6);
        let barrier = Arc::new(Barrier::new(6));

        for _ in 0..6 {
            let b = barrier.clone();
            handles.push(thread::spawn(move || {
                info!("before wait");
                b.wait();
                info!("after wait");
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn it_spawn_test02() {
        crate::init();
        let new_thread = thread::spawn(move || {
            thread::spawn(move || {
                loop {
                    info!("I am a new thread.");
                }
            });
        });

        new_thread.join().unwrap();
        info!("Child thread is finish!");

        thread::sleep(Duration::from_millis(1000));

    }

    #[test]
    fn it_thread_spawn_test() {
        crate::init();
        let v = vec![1,2,3];

        let handle = thread::spawn(move || {
            info!("Here's a vector: {:?}", v);
        });

        handle.join().unwrap();
    }

    #[test]
    fn it_unmoved_test() {
        crate::init();

        let handle = thread::spawn(|| {
            for i in 1..10 {
                info!("hi number {} from the spawned thread!", i);
                thread::sleep(Duration::from_millis(1));
            }
        });

        handle.join().unwrap();

        for i in 1..5 {
            info!("hi number {} from the main thread!", i);
            thread::sleep(Duration::from_millis(1));
        }

    }

    #[test]
    fn it_self_ref_test() {
        crate::init();
        let mut t = SelfRef::new("hello");
        t.init();
        info!("{}, {:p}", t.value(), t.pointer_to_value);

        t.value.push_str(", world");
        unsafe {
            (&mut *t.pointer_to_value).push_str("!");
        }

        info!("{}, {:p}", t.value(), t.pointer_to_value());
    }

    #[test]
    fn it_node_tree_test() {
        crate::init();
        let leaf = Rc::new(Node {
            value: 3,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![]),
        });

        info!("leaf strong = {}, weak = {}",
            Rc::strong_count(&leaf),
            Rc::weak_count(&leaf)
        );

        {
            let branch = Rc::new(Node {
                value: 5,
                parent: RefCell::new(Weak::new()),
                children: RefCell::new(vec![Rc::clone(&leaf)]),
            });

            *leaf.parent.borrow_mut() = Rc::downgrade(&branch);
            info!("branch strong = {}, weak = {}", 
                Rc::strong_count(&branch),
                Rc::weak_count(&branch)
            );
            info!("leaf strong = {}, weak = {}",
                Rc::strong_count(&leaf),
                Rc::weak_count(&leaf)
            );

        }

        info!("leaf parent = {:?}", leaf.parent.borrow().upgrade());
        info!("leaf strong = {}, weak = {}", 
            Rc::strong_count(&leaf),
            Rc::weak_count(&leaf)
        );
    }

    #[test]
    fn it_gadget_owner_test() {
        crate::init();
        let gadget_owner = Rc::new(
            Owner {
                name: "Gadget Man".to_string(),
                gadgets: RefCell::new(Vec::new()),
            }
        );

        // 创建工具，同时与主人进行关联：创建两个 gadget，他们分别持有 gadget_owner 的一个引用。
        let gadget1 = Rc::new(Gadget { id: 1, owner: gadget_owner.clone() });
        let gadget2 = Rc::new(Gadget { id: 2, owner: gadget_owner.clone() });

        // 为主人更新它所拥有的工具
        // 因为之前使用了 `Rc`，现在必须要使用 `Weak`，否则就会循环引用
        gadget_owner.gadgets.borrow_mut().push(Rc::downgrade(&gadget1));
        gadget_owner.gadgets.borrow_mut().push(Rc::downgrade(&gadget2));

        for gadget_opt in gadget_owner.gadgets.borrow().iter() {
            let gadget = gadget_opt.upgrade().unwrap();
            info!("Gadget {} owned by {}", gadget.id, gadget.owner.name);
        }

    }

    #[test]
    fn it_refcell_test() {
        crate::init();
        let mq = MsgQueue {
            msg_cache: RefCell::new(Vec::new()),
        };
        mq.send("Hello, world".to_string());
        mq.send("test2".to_string());
        info!("{:?}", mq.msg_cache.borrow());

        let s = Rc::new(RefCell::new("test".to_string()));
        let s1 = s.clone();
        let s2 = s.clone();
        s2.borrow_mut().push_str(",test2");
        info!("{}", s.borrow());
        info!("{}", s1.borrow());
        info!("{}", s2.borrow());

        let five = Rc::new(5);
        let weak_five = Rc::downgrade(&five);
        let strong_five = weak_five.upgrade();
        info!("{}", *strong_five.unwrap());

        drop(five);

        let strong_five = weak_five.upgrade();
        info!("{:?}", strong_five);

    }

    #[test]
    fn it_arc_test01() {
        crate::init();
        let s = Arc::new(String::from("多线程漫游者"));
        for _ in 0..10 {
            let s = Arc::clone(&s);
            let handle = thread::spawn(move || {
                info!("{}", s);
            });
            handle.join().unwrap();
        }

        let c = Cell::new("asdf");
        let one = c.get();
        c.set("qwer");
        let two = c.get();
        info!("{}, {}", one, two);

        let s = RefCell::new(String::from("hello, world"));
        {
            let mut s1 = s.borrow_mut();
            *s1 = String::from("test");
            info!("{}", s1);
        }
        {
            let s2 = s.borrow();
            info!("{}", s2);
        }

        let x = Cell::new(10);
        let y = &x;
        let z = &x;
        x.set(20);
        y.set(30);
        z.set(40);
        info!("{}", x.get());
        


    }

    #[test]
    fn it_rc_test() {
        crate::init();
        let gadget_owner = Rc::new(Owner {
            name: "Gadget Man".to_string(),
            gadgets: RefCell::new(Vec::new()),
        });

        let gadget1 = Gadget {
            id: 1,
            owner: Rc::clone(&gadget_owner),
        };
        let gadget2 = Gadget {
            id: 2,
            owner: Rc::clone(&gadget_owner),
        };

        drop(gadget_owner);
        info!("Gadget {} owner by {}", gadget1.id, gadget1.owner.name);
        info!("Gadget {} owner by {}", gadget2.id, gadget2.owner.name);
    }
}