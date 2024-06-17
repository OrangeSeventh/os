#![no_std]
#![no_main]
extern crate lib;
use lib::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
// 创建5个信号量作为筷子
static CHOPSTICKS: [Semaphore; 5] = semaphore_array![0, 1, 2, 3, 4];
// 创建服务生信号量
static WAITER: Semaphore = Semaphore::new(5);
fn main() -> isize {
    // 初始化每个筷子的信号量值为1
    for chopstick in CHOPSTICKS.iter() {
        chopstick.init(1);
    }
    WAITER.init(1);
    let mut philosopher_pids = [0; 5];
    // 创建5个哲学家线程
    for id in 0..5 {
        let pid = sys_fork();
        if pid == 0 {
            philosopher(id);
        } else {
            philosopher_pids[id] = pid;
        }
    }
    // 等待所有子进程退出
    for pid in philosopher_pids.iter() {
        if *pid != 0 {
            let exit_code = sys_wait_pid(*pid);
            println!("A philosopher exited with code {}", exit_code);
        }
    }
    0
}
fn philosopher(id: usize) -> ! {
    let mut rng = ChaChaRng::seed_from_u64(sys_get_pid() as u64);
    for _ in 0..20 {
        think(id, &mut rng);
        // 请求服务生允许就餐
        WAITER.wait();
        eat(id, &mut rng);
        // 报告服务生结束就餐
        WAITER.signal();
    }
    sys_exit(id as isize);
}
fn think(id: usize, rng: &mut ChaChaRng) {
    let duration = rng.gen_range(1..5);
    println!("Philosopher {} is thinking for {} seconds", id, duration);
    sleep(duration * 1000);
}
fn eat(id: usize, rng: &mut ChaChaRng) {
    let left_id = id;
    let right_id = (id + 1) % 5;
    let left_chopstick = &CHOPSTICKS[left_id];
    let right_chopstick = &CHOPSTICKS[right_id];
    // 尝试获取两根筷子
    left_chopstick.wait();
    println!(
        "Philosopher {} picked up left chopsticks(id:{})",
        id, left_id
    );
    right_chopstick.wait();
    println!(
        "Philosopher {} picked up right chopsticks(id:{})",
        id, right_id
    );
    // 进食
    let eat_duration = rng.gen_range(1..5);
    println!("Philosopher {} is eating for {} seconds", id, eat_duration);
    sleep(eat_duration * 1000);
    // 放下筷子
    left_chopstick.signal();
    println!(
        "Philosopher {} put down left chopsticks(id:{})",
        id, left_id
    );
    right_chopstick.signal();
    println!(
        "Philosopher {} put down right chopsticks(id:{})",
        id, right_id
    );
}
entry!(main);
