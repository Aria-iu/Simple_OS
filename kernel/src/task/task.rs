use crate::task::context::TaskContext;
#[derive(Copy,Clone)]
pub struct TaskControlBlock{
    pub taskstatus: TaskStatus,
    pub taskcontext: TaskContext,
}
#[derive(Copy,Clone,PartialEq)]
pub enum TaskStatus{
    UnInit,
    Ready,
    Running,
    Exited,
}