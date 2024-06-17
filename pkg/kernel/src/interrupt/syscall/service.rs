use core::alloc::Layout;

use crate::proc;
use crate::proc::ProcessContext;
use crate::utils::*;

use super::SyscallArgs;

pub fn spawn_process(args: &SyscallArgs) -> usize {
    // FIXME: get app name by args
    //       - core::str::from_utf8_unchecked
    //       - core::slice::from_raw_parts
    // FIXME: spawn the process by name
    // FIXME: handle spawn error, return 0 if failed
    // FIXME: return pid as usize
    let name_ptr = args.arg0 as *const u8;
    let name_len = args.arg1;

    let name = unsafe{
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(name_ptr, name_len))
    };
    match proc::spawn(name) {
        Some(pid) => pid.0 as usize,
        None => 0, // 如果进程创建失败，返回 0
    }
    // 0
}

pub fn sys_write(args: &SyscallArgs) -> usize {
    // FIXME: get buffer and fd by args
    //       - core::slice::from_raw_parts
    // FIXME: call proc::write -> isize
    // FIXME: return the result as usize
    let fd = args.arg0 as u8;
    let ptr = args.arg1 as *const u8;
    let len = args.arg2;

    let buf = unsafe { core::slice::from_raw_parts(ptr, len) };
    // write(fd, buf) as usize
    proc::write(fd, buf) as usize
}

pub fn sys_read(args: &SyscallArgs) -> usize {
    // FIXME: just like sys_write
    let fd = args.arg0 as u8;
    let ptr = args.arg1 as *mut u8;
    let len = args.arg2;

    let buf = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
    // read(fd ,buf) as usize
     proc::read(fd, buf) as usize
}

pub fn exit_process(args: &SyscallArgs, context: &mut ProcessContext) {
    // FIXME: exit process with retcode
    let retcode = args.arg0 as isize;
    // info!("exit_process: {}", retcode);
    proc::exit(retcode, context);
}

pub fn list_process() {
    // FIXME: list all processes
    // let processes = proc::print_process_list();
    // for p in processes {
    //     println!("Process ID: {}, Name: {}", p.pid(), p.name());
    // }
}

pub fn sys_allocate(args: &SyscallArgs) -> usize {
    let layout = unsafe { (args.arg0 as *const Layout).as_ref().unwrap() };

    if layout.size() == 0 {
        return 0;
    }

    let ret = crate::memory::user::USER_ALLOCATOR
        .lock()
        .allocate_first_fit(*layout);

    match ret {
        Ok(ptr) => ptr.as_ptr() as usize,
        Err(_) => 0,
    }
}

pub fn sys_deallocate(args: &SyscallArgs) {
    let layout = unsafe { (args.arg1 as *const Layout).as_ref().unwrap() };

    if args.arg0 == 0 || layout.size() == 0 {
        return;
    }

    let ptr = args.arg0 as *mut u8;

    unsafe {
        crate::memory::user::USER_ALLOCATOR
            .lock()
            .deallocate(core::ptr::NonNull::new_unchecked(ptr), *layout);
    }
}

pub fn sys_wait_pid(args: &SyscallArgs) -> usize {
    let pid = proc::ProcessId(args.arg0 as u16);
    let ret = proc::wait_pid(pid);
    ret as usize
}

pub fn sys_get_pid() -> u16 {
    proc::get_current_pid().0
}

pub fn sys_clock() -> i64 {
    if let Some(t) = clock::now() {
        return t.and_utc().timestamp_nanos_opt().unwrap_or_default();
    } else {
        return -1;
    }
}

pub fn sys_fork(context: &mut ProcessContext) {
    proc::fork(context);
}


pub fn sys_sem(args: &SyscallArgs, context: &mut ProcessContext) {
    match args.arg0 {
        0 => context.set_rax(proc::new_sem(args.arg1 as u32, args.arg2)),
        1 => context.set_rax(proc::remove_sem(args.arg1 as u32)),
        2 => proc::sem_signal(args.arg1 as u32, context),
        3 => proc::sem_wait(args.arg1 as u32, context),
        _ => context.set_rax(usize::MAX),
    }
}

pub fn sys_list_dir(args: &SyscallArgs) {
    let root = unsafe {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(args.arg0 as *const u8, args.arg1,))
    };
    crate::drivers::filesystem::ls(root);
}

pub fn sys_open(args: &SyscallArgs) -> usize {
    let path = unsafe {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(args.arg0 as *const u8, args.arg1,))
    };
    match proc::open(path, args.arg2 as u8) {
        Some(fd) => fd as usize,
        None => {
            warn!("sys_open failed, path: {}", path);
            0
        }
    }
}

pub fn sys_close(args: &SyscallArgs) -> usize {
    proc::close(args.arg0 as u8) as usize
}