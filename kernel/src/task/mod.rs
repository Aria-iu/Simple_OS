mod context;
mod switch;

#[allow(clippy::module_inception)]
mod task;

use crate::config::MAX_APP_NUM;
use crate::loader::{get_num_app, init_app_cx};
use crate::sync::UPSafeCell;
use self::switch::__switch;
use self::task::{TaskControlBlock,TaskStatus};
use crate::sbi::shut_down;
pub use context::TaskContext;

pub struct TaskManager{
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

struct TaskManagerInner{
    tasks:[TaskControlBlock;MAX_APP_NUM],
    cur_task: usize,
}

lazy_static!{
    pub static ref TASK_MANAGER:TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [
            TaskControlBlock{
                taskstatus: TaskStatus::UnInit,
                taskcontext: TaskContext::zero_init(),
            }
            ;MAX_APP_NUM];
        for (i,task) in tasks.iter_mut().enumerate(){
            task.taskcontext = TaskContext::goto_restore(init_app_cx(i));
            task.taskstatus = TaskStatus::Ready;
        }
        TaskManager{
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner{
                    tasks,
                    cur_task:0,
                })
            }
        }
    };
}

impl TaskManager{
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
