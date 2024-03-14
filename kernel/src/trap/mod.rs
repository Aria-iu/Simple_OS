mod context;
pub use context::TrapContext;

use core::arch::{asm, global_asm};
global_asm!(include_str!("trap.S"));
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

//use crate::{batch::run_next_app, syscall::syscall};
use crate::{config::{TRAMPOLINE, TRAP_CONTEXT}, syscall::syscall, task::*};
use crate::timer::set_next_trigger;
// 中断初始化
pub fn init(){
    set_kernel_trap_entry()
}

/// 开启时钟中断
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}
fn set_kernel_trap_entry(){
    unsafe{
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry(){
    unsafe{
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

// 弱化了 S 态–> S 态的 Trap 处理过程：直接 panic 。
#[no_mangle]
pub fn trap_from_kernel() -> !{
    panic!("a trap from kernel!")
}
// trap or syscall
#[no_mangle]
pub fn trap_handler() -> !{
    set_kernel_trap_entry();
    let cx = current_trap_cx();
    let scause = scause::read(); // get trap cause
    let stval = stval::read(); // get extra value
    match scause.cause(){
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.reg[10] = syscall(cx.reg[17], [cx.reg[10], cx.reg[11], cx.reg[12]]) as usize;
        },
        Trap::Exception(Exception::StoreFault) 
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) =>{
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
    // 在 trap_handler 完成 Trap 处理之后，
    // 我们需要调用 trap_return 返回用户态
    trap_return();
}


#[no_mangle]
pub fn trap_return() -> !{
    set_user_trap_entry();
    // 准备好 __restore 需要两个参数：
    // 分别是 Trap 上下文在应用地址空间中的虚拟地址和
    // 要继续执行的应用地址空间的 token 。
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();
    extern "C"{
        fn __alltraps();
        fn __restore();
    }
    // 跳转到 __restore
    // 计算 __restore 虚地址
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe{
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,    
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn)
        );
    }
    panic!("Unreachable in back_to_users");
}