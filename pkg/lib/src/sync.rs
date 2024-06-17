use core::{
    hint::spin_loop,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::*;

pub struct SpinLock {
    bolt: AtomicBool,
}

impl SpinLock {
    // 初始化锁，初始状态为`false`，表示未被持有
    pub const fn new() -> Self {
        Self {
            bolt: AtomicBool::new(false),
        }
    }

    pub fn acquire(&self) {
        // FIXME: acquire the lock, spin if the lock is not available
        while self
        .bolt
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err() {
            // 自旋等待
            spin_loop();
        }
    }

    pub fn release(&self) {
        // FIXME: release the lock
        // 将`bolt`的值设置回`false`，表示锁被释放
        self.bolt.store(false, Ordering::Release);
    }
}

// 允许`SpinLock`在多线程环境中安全地共享
unsafe impl Sync for SpinLock {} // Why? Check reflection question 5

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Semaphore {
    /* FIXME: record the sem key */
    key: u32,
}

impl Semaphore {
    pub const fn new(key: u32) -> Self {
        Semaphore { key }
    }

    #[inline(always)]
    pub fn init(&self, value: usize) -> bool {
        sys_new_sem(self.key, value)
    }
    
    /* FIXME: other functions with syscall... */

    #[inline(always)]
    pub fn free(&self) -> bool {
        sys_rm_sem(self.key)
    }

    #[inline(always)]
    pub fn signal(&self) {
        sys_sem_signal(self.key)
    }    

    #[inline(always)]
    pub fn wait(&self){
        sys_sem_wait(self.key)
    }
}

unsafe impl Sync for Semaphore {}

#[macro_export]
macro_rules! semaphore_array {
    [$($x:expr),+ $(,)?] => {
        [ $($crate::Semaphore::new($x),)* ]
    }
}
