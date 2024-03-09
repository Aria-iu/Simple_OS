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
struct USER_STACK{
    stack: [u8;USER_STACK_SIZE],
}

#[repr(align(4096))]
struct KERNEL_STACK{
    stack: [u8;KERNEL_STACK_SIZE],
}

impl USER_STACK{
    fn get_sp(&self) -> usize{
        self.stack.as_ptr() as usize + USER_STACK_SIZE
    }
}
impl KERNEL_STACK{
    fn get_sp(&self) -> usize{
        self.stack.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    fn push_context(&self, cx: TrapContext) -> &mut TrapContext{
        let cx_ptr = (self.get_sp - core::mem::sizeof::<TrapContext>()) as *mut TrapContext;
        unsafe{
            *cx_ptr = cx;
        }
        unsafe{cx_ptr.as_mut().unwrap()}
    }
}

pub struct AppManager{
    num_app: usize,
    cur_app: usize,
    app_start: [usize;MAX_APPS+1],
}

impl AppManager{
    pub fn get_current_app(&self) -> usize{
        self.cur_app
    }
    pub fn move_to_next_app(&mut self){
        self.cur_app += 1;
    }
    fn print_app_info(&self){
        println!("[kernel] num_app = {}",self.num_app);
        for i in 0..self.num_app{
            // this is why MAX_APPS+1 , to store both bgin and end
            println!("[kernel] app_{}[{:x},{:x}]",i,self.app_start[i],self.app_start[i+1]);
        }
    }
    fn load_app(&self, app_id: usize){
        
    }
}



//interfaces

pub fn init(){}

pub fn print_app_info(){}
// 现阶段在app执行结束或者产生fault时进行调用。
pub fn run_next_app()->!{}