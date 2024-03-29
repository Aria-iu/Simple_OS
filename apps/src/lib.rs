#![no_std]
#![feature(panic_info_message)]
#![feature(linkage)]

#[macro_use]
pub mod console;
pub mod syscall;
mod lang_items;


#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

#[no_mangle]
#[link_section=".text.entry"]
pub extern "C" fn _start() -> !{
    clear_bss();
    exit(main());
    panic!("Unreachabel after sys_exit in apps !");
}

fn clear_bss() {
    extern "C" {
        fn start_bss();
        fn end_bss();
    }
    (start_bss as usize..end_bss as usize).for_each(|addr| unsafe {
        (addr as *mut u8).write_volatile(0);
    });
}

use syscall::*;
pub fn write(fd: usize,buf: &[u8]) -> isize{
    sys_write(fd,buf)
}
pub fn exit(exit_code: i32) -> isize{
    sys_exit(exit_code)
}

pub fn get_time(ts: *mut TimeVal,_tz: usize) -> isize{
    sys_get_time(ts,_tz) as isize
}