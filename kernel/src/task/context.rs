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
    pub fn goto_restore(kstack_ptr: usize) -> Self{
        extern "C" {
            fn __restore();
        }
        Self{
            ra: __restore as usize,
            sp: kstack_ptr,
            reg: [0;12],
        }
    }
}