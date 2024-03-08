#![no_std]
#![no_main]

use core::arch::asm;
use riscv::register::sstatus::{self,SPP};
#[macro_use]
extern crate apps_lib;

#[no_mangle]
fn main() -> i32{
    println!("Try to access privileged CSR in U Mode");
    println!("Kernel should kill this application!");
    unsafe{
        // cannot do S Mode func in U mode
        sstatus::set_spp(SPP::User);
    }
    0
}