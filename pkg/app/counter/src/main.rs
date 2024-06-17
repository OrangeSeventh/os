#![no_std]
#![no_main]

use lib::*;

extern crate lib;

const THREAD_COUNT: usize = 8;
static mut COUNTER: isize = 0;

// 以下采用Semaphore保护的方式，使用SpinLock的部分均在注释中实现
// static mut MUTEX: SpinLock = SpinLock::new();
static SEM: Semaphore = Semaphore::new(1);

fn main() -> isize {
    let mut pids = [0u16; THREAD_COUNT];
    SEM.init(1);
    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            do_counter_inc();
            sys_exit(0);
        } else {
            pids[i] = pid; // only parent knows child's pid
        }
    }

    let cpid = sys_get_pid();
    println!("process #{} holds threads: {:?}", cpid, &pids);
    sys_stat();

    for i in 0..THREAD_COUNT {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }

    println!("COUNTER result: {}", unsafe { COUNTER });

    0
}

fn do_counter_inc() {
    for _ in 0..100 {
        // FIXME: protect the critical section
        // unsafe { MUTEX.acquire() };
        // inc_counter();
        // unsafe { MUTEX.release() };
        SEM.wait();
        inc_counter();
        SEM.signal();

    }
}

/// Increment the counter
///
/// this function simulate a critical section by delay
/// DO NOT MODIFY THIS FUNCTION
fn inc_counter() {
    unsafe {
        delay();
        let mut val = COUNTER;
        delay();
        val += 1;
        delay();
        COUNTER = val;
    }
}

#[inline(never)]
#[no_mangle]
fn delay() {
    for _ in 0..0x100 {
        core::hint::spin_loop();
    }
}

entry!(main);
// #![no_std]
// #![no_main]

// use core::sync::atomic::{AtomicIsize, Ordering};

// use lib::{entry, println, sys_exit, sys_fork, sys_get_pid, sys_stat, sys_wait_pid};

// extern crate lib;

// const THREAD_COUNT: usize = 8;
// static mut COUNTER: isize = 0;
// static SEM: Semaphore = Semaphore::new(1);

// pub struct Semaphore {
//     count: AtomicIsize,
// }

// impl Semaphore {
//     pub const fn new(count: isize) -> Self {
//         Semaphore {
//             count: AtomicIsize::new(count),
//         }
//     }

//     pub fn wait(&self) {
//         while self.count.fetch_sub(1, Ordering::SeqCst) <= 0 {
//             self.count.fetch_add(1, Ordering::SeqCst);
//             core::hint::spin_loop();
//         }
//     }

//     pub fn signal(&self) {
//         self.count.fetch_add(1, Ordering::SeqCst);
//     }
// }

// fn main() -> isize {
//     let mut pids = [0u16; THREAD_COUNT];

//     for i in 0..THREAD_COUNT {
//         let pid = sys_fork();
//         if pid == 0 {
//             do_counter_inc();
//             sys_exit(0);
//         } else {
//             pids[i] = pid;
//         }
//     }

//     let cpid = sys_get_pid();
//     println!("process #{} holds threads: {:?}", cpid, &pids);
//     sys_stat();

//     for i in 0..THREAD_COUNT {
//         println!("#{} waiting for #{}...", cpid, pids[i]);
//         sys_wait_pid(pids[i]);
//     }

//     println!("COUNTER result: {}", unsafe { COUNTER });

//     0
// }

// fn do_counter_inc() {
//     for _ in 0..100 {
//         SEM.wait();
//         inc_counter();
//         SEM.signal();
//     }
// }

// fn inc_counter() {
//     unsafe {
//         delay();
//         let mut val = COUNTER;
//         delay();
//         val += 1;
//         delay();
//         COUNTER = val;
//     }
// }

// #[inline(never)]
// #[no_mangle]
// fn delay() {
//     for _ in 0..0x100 {
//         core::hint::spin_loop();
//     }
// }

// entry!(main);