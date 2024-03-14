mod context;
mod switch;

#[allow(clippy::module_inception)]
mod task;

use crate::loader::{get_app_data, get_num_app};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use self::switch::__switch;
use self::task::{TaskControlBlock,TaskStatus};
use crate::sbi::shut_down;
use alloc::vec::Vec;
pub use context::TaskContext;

pub struct TaskManager{
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

struct TaskManagerInner{
    tasks:Vec<TaskControlBlock>,
    cur_task: usize,
}
/*
    在 TaskManagerInner 中 我 们 使 用 向 量 Vec 
    来 保 存 任 务 控 制 块。 在 全 局 任 务 管 理 器
    TASK_MANAGER 初始化的时候，
    只需使用 loader 子模块提供的 get_num_app 和 get_app_data 分
    别获取链接到内核的应用数量和每个应用的 ELF 文件格式的数据，
    然后依次给每个应用创建任务控制块并加入到向量中即可
*/
lazy_static!{
    pub static ref TASK_MANAGER:TaskManager = {
        println!("init TASK_MANAGER");
        let num_app = get_num_app();
        println!("num_app is {}",num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app{
            tasks.push(
                TaskControlBlock::new(get_app_data(i), i)
            )
        }
        TaskManager{
            num_app,
            inner:  unsafe {
                UPSafeCell::new(TaskManagerInner{
                    tasks,
                    cur_task:0, 
                })
            }
        }
    };
}

impl TaskManager{
    fn get_curren_token(&self) -> usize{
        let inner = self.inner.exclusive_access();
        let current = inner.cur_task;
        inner.tasks[current].get_user_token()
    }
    
    fn get_current_trap_cx(&self) -> &mut TrapContext{
        let inner = self.inner.exclusive_access();
        let current = inner.cur_task;
        inner.tasks[current].get_trap_cx()
    }

    fn run_first_task(&self){
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.taskstatus = TaskStatus::Running;
        let next_task_cx_ptr = &task0.taskcontext as *const TaskContext;
        drop(inner);

        let mut unused = TaskContext::zero_init();

        unsafe{
            __switch(&mut unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    fn mark_current_suspend(&self){
        let mut inner = self.inner.exclusive_access();
        let current = inner.cur_task;
        inner.tasks[current].taskstatus = TaskStatus::Ready;
    }

    fn mark_current_exited(&self){
        let mut inner = self.inner.exclusive_access();
        let current = inner.cur_task;
        inner.tasks[current].taskstatus = TaskStatus::Exited;
    }

    fn find_next_task(&self) -> Option<usize>{
        let inner = self.inner.exclusive_access();
        let current = inner.cur_task;
        (current + 1..current+self.num_app+1)
        .map(|id| id % self.num_app)
        .find(|id| inner.tasks[*id].taskstatus == TaskStatus::Ready)
    }

    pub fn change_current_program_brk(&self,size: i32) -> Option<usize>{
        let mut inner = self.inner.exclusive_access();
        let cur = inner.cur_task;
        inner.tasks[cur].change_program_brk(size)
    }

    fn run_next_task(&self){
        if let Some(next) = self.find_next_task(){
            let mut inner = self.inner.exclusive_access();
            let current = inner.cur_task;
            inner.tasks[next].taskstatus = TaskStatus::Running;
            inner.cur_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].taskcontext as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].taskcontext as *const TaskContext;
            drop(inner);
            unsafe{
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }

        }else{  
            println!("All applications completed!");
            shut_down();
        }
    }

}
// 通过 current_user_token 和 current_trap_cx 
// 分别可以获得当前正在执行的应用的地址空间的 token
// 和可以在内核地址空间中修改位于该应用地址空间中的 Trap 上下文的可变引用。
pub fn current_user_token() -> usize{
    TASK_MANAGER.get_curren_token()
}

pub fn current_trap_cx() -> &'static mut TrapContext{
    TASK_MANAGER.get_current_trap_cx()
}
 
/// run first task
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// rust next task
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// suspend current task
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspend();
}

/// exit current task
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// suspend current task, then run next task
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// exit current task,  then run next task
/// 当程序i运行结束，访问无效i指令引起trap，将程序标记为exited，然后run next
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

pub fn change_program_brk(size: i32) -> Option<usize>{
    TASK_MANAGER.change_current_program_brk(size)
}
