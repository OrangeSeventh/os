#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() -> isize {
    // println!("Hello, world!!!");

    // 233
    println!("Hello, world!");
    sleep(5000);
    println!("Sleep 5s has done!");
    return 0;
}

entry!(main);
