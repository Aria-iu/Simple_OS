use crate::batch::run_next_app;
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal{
    pub sec: usize,
    pub usec: usize,
}

pub fn sys_exit(exit_code: i32) -> isize{
    println!("[kernel] Application exited with code {}", exit_code);
    // this is in batch system we can do
    run_next_app()
}

pub fn sys_yield() -> !{
    loop {}
}

pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize{
    let us = get_time_us();

    unsafe{
        *ts = TimeVal{
            sec: us/1_000_000,
            usec: us%1_000_000,
        }
    }
    0
}