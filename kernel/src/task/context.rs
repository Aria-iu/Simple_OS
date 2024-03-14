use crate::trap::trap_return;

#[derive(Copy,Clone)]
#[repr(C)]
pub struct TaskContext{
    ra : usize,
    sp : usize,
    // s0..s11
    reg: [usize;12],
}

impl TaskContext{
    pub fn zero_init() -> Self{
        Self{
            ra: 0,
            sp: 0,
            reg: [0;12],
        }
    }
    // set Task Context{__restore ASM funciton: trap_return, 
    // sp: kstack_ptr, s: s_0..12}
    pub fn goto_trap_return(kstack_ptr: usize) -> Self{
        Self{
            ra: trap_return as usize,
            reg: [0;12],
            sp : kstack_ptr,
        }
    }
}