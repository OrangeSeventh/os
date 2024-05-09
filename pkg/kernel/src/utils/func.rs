pub fn 
test() -> ! {
    let mut count = 0;
    let id;
    if let Some(id_env) = crate::proc::env("id") {
        id = id_env
    } else {
        id = "unknown".into()
    }
    loop {
        // TODO: better way to show more than one process is running?
        count += 1;
        if count == 1000 {
            count = 0;
            print!("\r{:-6} => Tick!", id);
        }
        unsafe {
            x86_64::instructions::hlt();
        }
    }
}

#[inline(never)]
fn huge_stack() {
    println!("Huge stack testing...");

    // let mut stack: [u64; 4096] = [0u64; 0x1000];
    let mut stack= [0u64; 128];

    println!("creating stack...");
    // for (idx, item) in stack.iter_mut().enumerate() {
    //     *item = idx as u64;
    // }
    for i in 0..stack.len() {
        stack[i] = i as u64;
        println!("{:#05x} == {:#05x}", i, stack[i]);
    }
    // for i in 0..stack.len() / 256 {
    //     println!("{:#05x} == {:#05x}", i * 256, stack[i * 256]);
    // }
}

pub fn stack_test() -> ! {
    huge_stack();
    info!("f");
    crate::proc::process_exit(0);
}
