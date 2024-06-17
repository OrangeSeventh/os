#![no_std]
#![no_main]

extern crate lib;
use lib::*;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;

static mut MUTEX: SpinLock = SpinLock::new();
static mut fish: [char; 4] = [' ', ' ', ' ', ' '];
fn main() -> isize {
    let mut pids = [0; 3];
    let mut rng = ChaChaRng::seed_from_u64(sys_time_u64() as u64);
    // 创建三个子进程
    for i in 0..3 {
        let pid = sys_fork();
        if pid == 0 {
            match i {
                0 => loop {
                    unsafe {
                        MUTEX.acquire();
                        if fish[0] == ' ' {
                            //随机取>或者<
                            let tmp = if rng.gen_range(0..2) == 0 { '>' } else { '<' };
                            fish[0] = tmp;
                            print!("{}", tmp);
                        } else if fish[1] != ' ' && fish[2] == ' ' {
                            let tmp = if fish[1] == '>' { '<' } else { '>' };
                            fish[2] = tmp;
                            print!("{}", tmp);
                        } else if fish[3] != ' ' {
                            let tmp = if rng.gen_range(0..2) == 0 { '>' } else { '<' };
                            fish[0] = tmp;
                            fish[1] = ' ';
                            fish[2] = ' ';
                            fish[3] = ' ';
                            print!("{}", tmp);
                        }
                        MUTEX.release();
                    }
                    sleep(1)
                },
                1 => loop {
                    unsafe {
                        MUTEX.acquire();
                        if fish[0] != ' ' && fish[1] == ' ' {
                            let tmp = if fish[0] == '>' { '<' } else { '>' };
                            fish[1] = tmp;
                            print!("{}", tmp);
                        }
                        MUTEX.release();
                    }
                    sleep(1)
                },
                2 => loop {
                    unsafe {
                        MUTEX.acquire();
                        if fish[2] != ' ' && fish[3] == ' ' {
                            fish[3] = '_';
                            print!("_");
                        }
                        MUTEX.release();
                    }
                    sleep(1)
                },
                _ => {}
            }
            sys_exit(0);
        } else {
            pids[i] = pid;
        }
    }

    // 父进程等待所有子进程退出
    for pid in pids {
        if pid != 0 {
            sys_wait_pid(pid);
        }
    }

    0
}
entry!(main);
