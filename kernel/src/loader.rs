
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use core::arch::asm;

const MAX_APPS: usize = 16;
const USER_STACK_SIZE: usize = 4096 * 2;    // 8K
const KERNEL_STACK_SIZE: usize = 4096 * 2;

// for apps settings
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

#[repr(align(4096))]
#[derive(Copy, Clone)]
pub struct UserStack{
    stack: [u8;USER_STACK_SIZE],
}

#[repr(align(4096))]
#[derive(Copy, Clone)]
struct KernelStack{
    stack: [u8;KERNEL_STACK_SIZE],
}
static KERNEL_STACK: [KernelStack; MAX_APPS] = [KernelStack {
    stack: [0; KERNEL_STACK_SIZE],
}; MAX_APPS];

static USER_STACK: [UserStack; MAX_APPS] = [UserStack {
    stack: [0; USER_STACK_SIZE],
}; MAX_APPS];
/* 
static KERNEL_STACK: KernelStack = KernelStack {
    stack: [0; KERNEL_STACK_SIZE],
};
pub static USER_STACK: UserStack = UserStack {
    stack: [0; USER_STACK_SIZE],
};
*/
impl UserStack{
    pub fn get_sp(&self) -> usize{
        self.stack.as_ptr() as usize + USER_STACK_SIZE
    }
}
impl KernelStack{
    fn get_sp(&self) -> usize{
        self.stack.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    fn push_context(&self, trap_cx: TrapContext) -> usize{
        let trap_cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe{
            *trap_cx_ptr = trap_cx;
        }
        trap_cx_ptr as usize
    }
}

/// Get base address of app i.
fn get_base_i(app_id: usize) -> usize {
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

pub fn get_num_app() -> usize{
    extern "C" {
        fn _num_app();
    }
    unsafe{((_num_app) as usize as *const usize).read_volatile()}
}

pub fn load_app(){
    extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    // load apps
    for i in 0..num_app {
        let base_i = get_base_i(i);
        // clear region
        (base_i..base_i + APP_SIZE_LIMIT)
            .for_each(|addr| unsafe { (addr as *mut u8).write_volatile(0) });
        // load app from data section to memory
        let src = unsafe {
            core::slice::from_raw_parts(app_start[i] as *const u8, app_start[i + 1] - app_start[i])
        };
        let dst = unsafe { core::slice::from_raw_parts_mut(base_i as *mut u8, src.len()) };
        dst.copy_from_slice(src);
    }
    unsafe{asm!("fence.i");}
}

/// get app info with entry and sp and save `TrapContext` in kernel stack
pub fn init_app_cx(app_id: usize) -> usize {
    KERNEL_STACK[app_id].push_context(
        TrapContext::app_init_context(
        get_base_i(app_id),
        USER_STACK[app_id].get_sp(),
    ))
}