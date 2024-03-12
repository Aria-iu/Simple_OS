use crate::task::context::TaskContext;
use core::arch::global_asm;

global_asm!(include_str!("switch.S"));
// s0 - s11 saved regs
extern "C" {
    /// Switch to the context of `next_task_cx_ptr`, 
    /// saving the current context
    /// in `current_task_cx_ptr`.
    pub fn __switch(current_task_cx_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}