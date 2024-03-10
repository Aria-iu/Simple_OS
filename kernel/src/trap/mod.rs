mod context;
pub use context::TrapContext;

use core::arch::global_asm;
global_asm!(include_str!("trap.S"));
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Trap},
    stval, stvec,
};

use crate::{batch::run_next_app, syscall::syscall};

// set trap settings
pub fn init(){
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}
// trap or syscall
#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext)  -> &mut TrapContext{
    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
    match scause.cause(){
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.reg[10] = syscall(cx.reg[17], [cx.reg[10], cx.reg[11], cx.reg[12]]) as usize;
        },
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) =>{
            println!("[kernel] PageFault in application, kernel killed it.");
            run_next_app();
        },
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            run_next_app();
        },
        _=>{
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    cx
}