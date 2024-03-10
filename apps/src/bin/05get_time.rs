#![no_std]
#![no_main]

#[macro_use]
extern crate apps_lib;
use apps_lib::syscall::sys_get_time;

use crate::apps_lib::syscall::TimeVal;

#[no_mangle]
fn main() -> i32{
    println!("Try to call sys_get_time");
    let mut tv = TimeVal{sec:0,usec:0};
    let _tz: usize = 0;
    sys_get_time( &mut tv, _tz);
    println!("{:?}",tv);
    0
}