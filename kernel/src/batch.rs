// 在trap时实现用户栈和内核栈的转换（trap.S）

use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use lazy_static::*; 

const MAX_APPS: usize = 16;
const USER_STACK_SIZE: usize = 4096 * 2;    // 8K
const KERNEL_STACK_SIZE: usize = 4096 * 2;

// for apps settings
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

#[repr(align(4096))]
struct UserStack{
    stack: [u8;USER_STACK_SIZE],
}

#[repr(align(4096))]
struct KernelStack{
    stack: [u8;KERNEL_STACK_SIZE],
}

static KERNEL_STACK: KernelStack = KernelStack {
    stack: [0; KERNEL_STACK_SIZE],
};
static USER_STACK: UserStack = UserStack {
    stack: [0; USER_STACK_SIZE],
};

impl UserStack{
    fn get_sp(&self) -> usize{
        self.stack.as_ptr() as usize + USER_STACK_SIZE
    }
}
impl KernelStack{
    fn get_sp(&self) -> usize{
        self.stack.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    fn push_context(&self, cx: TrapContext) -> &mut TrapContext{
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
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
    pub fn print_app(&self){
        println!("[kernel] num_app = {}",self.num_app);
        for i in 0..self.num_app{
            // this is why MAX_APPS+1 , to store both bgin and end
            println!("[kernel] app_{}[{:x},{:x}]",i,self.app_start[i],self.app_start[i+1]);
        }
    }
    pub unsafe fn load_app(&self, app_id: usize){
        if app_id >= MAX_APPS {
            println!("Apps Complete!");
            #[cfg(feature = "board_qemu")]
            use crate::board::QEMUExit;
            #[cfg(feature = "board_qemu")]
            crate::board::QEMU_EXIT_HANDLE.exit_success();
        }
        println!("[Kernel] load app_{}",app_id);
        // 加载app的数据，到APP_BASE_ADDRESS
        core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8,APP_SIZE_LIMIT).fill(0);
        let app_src = core::slice::from_raw_parts_mut(
            self.app_start[app_id] as *mut u8,
            self.app_start[app_id+1] - self.app_start[app_id]
        );
        let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8,app_src.len());
        // OK loaded
        app_dst.copy_from_slice(app_src);
    }
}

 
lazy_static!{
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe{
        UPSafeCell::new(
            {
                extern "C" {fn _num_app();}
                let num_app_ptr = _num_app as usize as *mut usize;
                let num_app = num_app_ptr.read_volatile();
                let mut app_start: [usize;MAX_APPS+1] = [0;MAX_APPS+1];
                let app_start_raw: &[usize] = core::slice::from_raw_parts_mut(num_app_ptr.add(1),num_app+1);
                app_start[..=num_app].copy_from_slice(app_start_raw);
                AppManager{
                    num_app,
                    cur_app: 0,
                    app_start,
                }
            }
        )
    };
}



//interfaces

pub fn init(){
    print_app_info();
}

pub fn print_app_info(){
    // lazy_static 的 APPMANAGER 执行输出
    APP_MANAGER.exclusive_access().print_app();
}

// 现阶段在app执行结束或者产生fault时进行调用。
pub fn run_next_app()->!{
    // 加载下一个app，通过设置上下文调用__restore，返回用户态执行
    let mut app_manager = APP_MANAGER.exclusive_access();
    let cur_app = app_manager.get_current_app();
    unsafe{
        app_manager.load_app(cur_app);
    }
    app_manager.move_to_next_app();
    drop(app_manager);

    extern "C"{
        fn __restore(cx_addr: usize);
    }
    unsafe{
        __restore(KERNEL_STACK.push_context(
            TrapContext::app_init_context(
                APP_BASE_ADDRESS,
                USER_STACK.get_sp(),
                )
            ) as *const _ as usize
        );
    }
    panic!("Unreachable in batch::run_current_app!");

}