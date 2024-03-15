#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
extern crate bitflags;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod config;
mod lang_items;
mod loader;
mod mm;
mod sbi;
mod sync;
pub mod syscall;
pub mod task;
mod timer;
pub mod trap;

core::arch::global_asm!(include_str!("entry.asm"));
core::arch::global_asm!(include_str!("link_app.S"));

/// clear BSS segment
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

#[no_mangle]
/// the rust entry-point of os
pub fn rust_main() -> ! {
    welcome();
    clear_bss();
    println!("[kernel] Hello, world!");
    mm::init();
    println!("[kernel] back to world!");
    mm::remap_test();
    trap::init();
    //trap::enable_interrupt();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    loader::list_apps();
    task::add_init_proc();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}

fn welcome() {
    println!("                                                                                ,----..                 ");
    println!("\x1b[31m  .--.--.                      ____              ,--,                          /   /   \\   .--.--.      \x1b[0m");
    println!(" /  /    '.   ,--,           ,'  , `.,-.----.  ,--.'|                         /   .     : /  /    '.    ");
    println!("\x1b[31m|  :  /`. / ,--.'|        ,-+-,.' _ |\\    /  \\ |  | :                        .   /   ;.  \\  :  /`. /    \x1b[0m");
    println!(";  |  |--`  |  |,      ,-+-. ;   , |||   :    |:  : '                       .   ;   /  ` ;  |  |--`     ");
    println!("|  :  ;_    `--'_     ,--.'|'   |  |||   | .\\ :|  ' |      ,---.            ;   |  ; \\ ; |  :  ;_       ");
    println!("\x1b[31m \\  \\    `. ,' ,'|   |   |  ,', |  |,.   : |: |'  | |     /     \\           |   :  | ; | '\\  \\    `.    \x1b[0m");
    println!("  `----.   \\'  | |   |   | /  | |--' |   |  \\ :|  | :    /    /  |          .   |  ' ' ' : `----.   \\   ");
    println!("\x1b[31m  __ \\  \\  ||  | :   |   : |  | ,    |   : .  |'  : |__ .    ' / |          '   ;  \\; /  | __ \\  \\  |   \x1b[0m");
    println!(" /  /`--'  /'  : |__ |   : |  |/     :     |`-'|  | '.'|'   ;   /|        ___\\   \\  ',  / /  /`--'  /   ");
    println!("\x1b[31m'--'.     / |  | '.'||   | |`-'      :   : :   ;  :    ;'   |  / |     .'  .`|;   :    / '--'.     /    \x1b[0m");
    println!("  `--'---'  ;  :    ;|   ;/          |   | :   |  ,   / |   :    |  .'  .'   : \\   \\ .'    `--'---'     ");
    println!("            |  ,   / '---'           `---'.|    ---`-'   \\   \\  /,---, '   .'   `---`                   ");
    println!("             ---`-'                    `---`              `----' ;   |  .'                              ");
    println!("                                                                 `---'                                  ");
}
