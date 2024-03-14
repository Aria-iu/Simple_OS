use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next};
//use crate::batch::run_next_app;
use crate::timer::get_time_ms;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal{
    pub sec: usize,
    pub usec: usize,
}

pub fn sys_exit(exit_code: i32) -> !{
    println!("[kernel] Application exited with code {}", exit_code);
    // this is in batch system we can do
    //run_next_app()
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize{
    //loop {}
    suspend_current_and_run_next();
    0
}

/// get time in milliseconds
pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}