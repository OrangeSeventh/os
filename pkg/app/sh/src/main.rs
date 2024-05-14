#![no_std]
#![no_main]

use lib::{entry, print, println, stdin, sys_exit, sys_list_app, sys_spawn, sys_stat, sys_wait_pid, vec::Vec};

extern crate lib;

/// Entry point for the shell
fn main() -> isize {
    println!("Orange_Seventh's YSOS Shell: Welcome! Type 'help' for assistance.");
    loop {
        print!("> ");
        let command = stdin().read_line(); 
        let line: Vec<&str> = command.trim().split_whitespace().collect();  // 使用 split_whitespace 以自动处理多余的空格
        match line.first().map(|s| *s) {  // 使用 first() 获取第一个命令，确保不会panic如果输入为空
            Some("list") => list_apps(),
            Some("process") => stat_processes(),
            Some("spawn") => spawn_app(line),
            Some("help") => print_help(),
            Some("exit") => sys_exit(0),
            _ => println!("Unknown command. Type 'help' for assistance."),
        }
    }
    0
}

/// Lists all user apps available
fn list_apps() {
    println!("Available applications:");
    sys_list_app();
}

// Lists all current processes
fn stat_processes() {
    println!("Current processes:");
    sys_stat();
}

/// Runs a user application
fn spawn_app(line: Vec<&str>) {
    if line.len() < 2 {
        println!("Your command is missing an application name. Type 'help' for assistance.");
        return ;
    }
    let app_name = line[1];
    sys_spawn(app_name);
}

/// Prints help information
fn print_help() {
    println!("YSOS Shell Help:");
    println!("list  - Lists all available applications");
    println!("process  - Lists all current processes");
    println!("spawn <app_name>   - Runs a specified application");
    println!("help  - Displays this help message");
    println!("exit  - Exits the shell");
}

entry!(main);
