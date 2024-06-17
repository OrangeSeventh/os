#![no_std]
#![no_main]

use alloc::vec::Vec;
use lib::{entry, format, print, println, stdin, string::String, sys_close, sys_exit, sys_list_app, sys_list_dir, sys_open, sys_read, sys_spawn, sys_stat, sys_wait_pid};
extern crate alloc;
extern crate lib;

/// Entry point for the shell
fn main() -> isize {
    println!("Orange_Seventh's YSOS Shell: Welcome! Type 'help' for assistance.");
    loop {
        print!("> ");
        let command = stdin().read_line(); 
        let line: Vec<&str> = command.trim().split_whitespace().collect();  // 使用 split_whitespace 以自动处理多余的空格
        match line.first().map(|s| *s) {  // 使用 first() 获取第一个命令，确保不会panic如果输入为空
            Some("app") => list_apps(),
            Some("process") => stat_processes(),
            Some("spawn") => spawn_app(line),
            Some("ls") => list(&line),
            Some("cat") => cat(line),
            Some("help") => print_help(),
            Some("exit") => sys_exit(0),

            _ => println!("Unknown command. Type 'help' for assistance."),
        }
    }
    // 0
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

/// list files in the specified directory
fn list(line: &[&str]) {
    if line.len() < 2 {
        println!("Your command is missing a file name. Type 'help' for assistance.");
        return ;
    }
    let dir = line[1];
    sys_list_dir(dir);
}

/// Displays the contents of a specified file
fn cat(line: Vec<&str>) {
    if line.len() < 2 {
        println!("Your command is missing a file name. Type 'help' for assistance.");
        return ;
    }
    let path = line[1];
    let root_dir = "/";
    let path = if path.starts_with('/') {
        String::from(path)
    } else {
        format!("{}{}", root_dir, path)
    }
    .to_ascii_uppercase();
    // println!("Attempting to open file: {}", path);
    let fd = sys_open(path.as_str(), lib::FileMode::ReadOnly);
    // println!("Attempting to open the fd: {}", fd);
    if fd == 0 {
        println!("File not found or cannot open");
        return ;
    }

    let mut buf = lib::vec![0; 0x4000];
    let size = sys_read(fd, &mut buf);
    
    if size.is_none() {
        println!("Failed to read file");
        return ;
    }
    
    let size = size.unwrap();
    if size == 0 {
        println!("File is empty or buffer is too small");
        return ;
    }
    print!("{}", core::str::from_utf8(&buf[..size]).unwrap());
    sys_close(fd);
}

/// Prints help information
fn print_help() {
    println!("YSOS Shell Help:");
    println!("app  - Lists all available applications");
    println!("process  - Lists all current processes");
    println!("spawn <app_name>   - Runs a specified application");
    println!("ls <dir_name>  - Lists all files in the specified directory");
    println!("cat <file_name>  - Displays the contents of a specified file");
    println!("help  - Displays this help message");
    println!("exit  - Exits the shell");
}

entry!(main);
