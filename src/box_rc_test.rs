#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
#![allow(dead_code)]
#![allow(unused_variables)]
use core::borrow;
use std::{cell::{RefCell, UnsafeCell}, marker::PhantomPinned, pin::Pin, ptr::NonNull, rc::{Rc, Weak}};


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


#[cfg(test)]
mod tests {
    use std::{cell::{Cell, RefCell}, rc::Rc, sync::{Arc, Barrier}, thread, time::Duration};
    use log::info;
    use super::*;

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