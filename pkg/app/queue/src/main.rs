#![no_std]
#![no_main]

use lib::*;
extern crate lib;

// 消息队列的最大容量
const QUEUE_CAPACITY: usize = 8;
// 静态变量用于跟踪消息队列中的消息数量
static mut MESSAGE_COUNT: usize = 0;

// 信号量
static SEM_FULL: Semaphore = Semaphore::new(1); // 控制队列不满
static SEM_EMPTY: Semaphore = Semaphore::new(2); // 控制队列不空
static MUTEX: Semaphore = Semaphore::new(3);
fn main() -> isize {
    // 初始化信号量
    SEM_FULL.init(QUEUE_CAPACITY);
    SEM_EMPTY.init(0);
    MUTEX.init(1);
    let mut pids = [0u16; 16];
    // 创建子进程
    for i in 0..16 {
        let pid = sys_fork();
        if pid == 0 {
            // 子进程
            if i < 8 {
                // 前8个是生产者
                producer(i);
            } else {
                // 后8个是消费者
                consumer(i);
            }
            sys_exit(0); // 子进程完成任务后退出
        }else {
            pids[i] = pid;
        }
    }
    // 输出当前进程信息
    sys_stat();
    // 父进程等待所有子进程
    for i in 0..16 {
        let id = sys_wait_pid(pids[i]);
        println!("Pid:{} exit ret {}.",pids[i], id);
    }

    // 检查并输出最终的消息队列状态
    unsafe {
        let remaining_messages = MESSAGE_COUNT;
        println!("Remaining messages in queue: {}", remaining_messages);
    }
    0
}

// 生产者函数
fn producer(id: usize) {
    for _ in 0..10 {
        // 等待队列不满
        SEM_FULL.wait();
        MUTEX.wait();
        // 生产消息（这里简化为增加计数）
        unsafe {
            MESSAGE_COUNT += 1;
            println!(

                "Producer {} produced a message. Total messages: {}",
                id, MESSAGE_COUNT
            );
        };
        
        MUTEX.signal();
        // sys_stat();

        // 信号队列不空
        SEM_EMPTY.signal();
    }
}
// 消费者函数
fn consumer(id: usize) {
    for _ in 0..10 {
        // 等待队列不空
        SEM_EMPTY.wait();
        MUTEX.wait();
        // 消费消息（这里简化为减少计数）
        unsafe {
            MESSAGE_COUNT -= 1;
            println!(
                "Consumer {} consumed a message. Remaining messages: {}",
                id, MESSAGE_COUNT
            );
        }
        
        MUTEX.signal();
        // sys_stat();

        // 信号队列不满
        SEM_FULL.signal();
    }
}

entry!(main);

// const THREAD_COUNT: usize = 16;
// static mut MESSAGES: isize = 0;

// static SEM_EMPTY: Semaphore = Semaphore::new(0);
// static SEM_FULL: Semaphore = Semaphore::new(1);
// static mut SEMAPHORE: Semaphore = Semaphore::new(2);

// fn main() {
//     let mut pids = [0u16; THREAD_COUNT];
//     unsafe {
//         SEMAPHORE.init(1);
//         SEM_EMPTY.init(THREAD_COUNT * 2);
//         SEM_FULL.init(0);
//     }
//     for i in 0..THREAD_COUNT {
//         let pid = sys_fork();
//         if pid == 0 {
//             if i% 2 == 0 {
//                 for _ in 0..10 {
//                     producer();
//                 }
//             } else {
//                 for _ in 0..10 {
//                     consumer();
//                 }
//             }
//             sys_exit(0);
//         } else {
//             pids[i] = pid;
//         }
//     }
//     let cpid = sys_get_pid();
//     println!("process #{} holds threads: {:?}", cpid, &pids);

//     for i in 0..THREAD_COUNT {
//         sys_wait_pid(pids[i]);
//         println!("{} exit, {} remain", pids[i], unsafe{MESSAGES});
//     }
//     println!("All threads exit, {} remain", unsafe{MESSAGES});
// }

// // 生产者函数
// fn producer() {
//     unsafe {
//         SEM_EMPTY.wait();
//         SEMAPHORE.wait();
//         MESSAGES += 1;
//         println!("Total messages: {}", MESSAGES);
//         SEMAPHORE.signal();
//         SEM_FULL.signal();
//     }
// }
// // 消费者函数
// fn consumer() {
//     unsafe {
//         SEM_FULL.wait();
//         SEMAPHORE.wait();
//         MESSAGES += 1;
//         println!("Total messages: {}", MESSAGES);
//         SEMAPHORE.signal();
//         SEM_EMPTY.signal();
//     }
// }