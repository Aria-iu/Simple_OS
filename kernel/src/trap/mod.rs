mod context;
pub use context::TrapContext;

use core::arch::global_asm;
global_asm!(include_str!("trap.S"));
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

//use crate::{batch::run_next_app, syscall::syscall};
use crate::{syscall::syscall, task::*};
use crate::timer::set_next_trigger;
// 中断初始化
pub fn init(){
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

/// 开启时钟中断
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
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
            //run_next_app();
            exit_current_and_run_next();
        },
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            //run_next_app();
            exit_current_and_run_next();
        },
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
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