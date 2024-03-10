#![no_std]
#![no_main]
#![feature(panic_info_message)]

#[macro_use]
mod console;
use core::arch::global_asm;
mod lang_items;
mod sbi;
pub mod trap;
pub mod batch;
mod sync;
pub mod syscall;
pub mod timer;
pub mod config;

#[macro_use]
extern crate lazy_static;

#[cfg(feature = "qemu")]
#[path = "../board/qemu.rs"]
mod board;

global_asm!(include_str!("entry.S"));
global_asm!(include_str!("link_app.S"));


fn clear_bss(){
    extern "C"{
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe {
        (a as *mut u8).write_volatile(0)
    });
}

#[no_mangle]
fn simpl_os_main() -> ! {
    extern "C"{
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn boot_stack(); // stack bottom
        fn boot_stack_top(); // stack top
    }
    clear_bss();
    welcome();
    println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
    println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
    println!(
        "boot_stack [{:#x}, {:#x})",
        boot_stack as usize, boot_stack_top as usize
    );
    println!(".bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
    println!("begin run some Apps here!");
    trap::init();
    batch::init();
    batch::run_next_app();

    #[cfg(feature = "qemu")]
    use crate::board::QEMUExit;

    #[cfg(feature="qemu")]
    crate::board::QEMU_EXIT_HANDLE.exit_success();

    #[cfg(feature = "board_k210")]
    panic!("Unreachable in rust_main!");

    panic!("Should Shutdown!");
}

fn welcome(){
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