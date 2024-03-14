use crate::{config::{kernel_stack_position, TRAP_CONTEXT}, mm::{MapPermission, MemorySet, PhysPageNum, VirtAddr, KERNEL_SPACE}, task::context::TaskContext, trap::{trap_handler, TrapContext}};


pub struct TaskControlBlock{
    pub taskstatus: TaskStatus,
    pub taskcontext: TaskContext,

    // come to address
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    pub heap_bottom: usize,
    pub program_brk: usize,
}
#[derive(Copy,Clone,PartialEq)]
pub enum TaskStatus{
    UnInit,
    Ready,
    Running,
    Exited,
}

impl TaskControlBlock{
    pub fn get_trap_cx(&self) -> &'static mut TrapContext{
        self.trap_cx_ppn.get_mut()
    }
    
    pub fn new(elf_data:&[u8],app_id: usize) -> Self{
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set.
            translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let task_status = TaskStatus::Ready;
        // map a kernel stack in kernel space
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        // 通过insert_framed_area 实际将这个逻辑段加入到内核地址空间中
        KERNEL_SPACE.exclusive_access()
            .insert_framed_area(
                kernel_stack_bottom.into(), 
                kernel_stack_top.into(),
                 MapPermission::R|MapPermission::W)
            ;
        // 在应用的内核栈顶压入一个跳转到 trap_return 而不是 __restore 的任务上下文，这
        // 主要是为了能够支持对该应用的启动并顺利切换到用户地址空间执行。
        let task_control_block = Self{
            taskstatus :task_status,
            taskcontext : TaskContext::goto_trap_return(kernel_stack_top),
            memory_set: memory_set,
            trap_cx_ppn: trap_cx_ppn,
            base_size: user_sp,
            heap_bottom: user_sp,
            program_brk: user_sp,
        };
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point, 
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
            );
        task_control_block
    }

    pub fn change_program_brk(&mut self,size:i32) -> Option<usize>{
        let old_break = self.program_brk;
        let new_break = self.program_brk as isize + size as isize;
        let result = if size < 0 {
            self.memory_set
                .shrink_to(VirtAddr(self.heap_bottom), VirtAddr(new_break as usize))
        } else {
            self.memory_set
                .append_to(VirtAddr(self.heap_bottom), VirtAddr(new_break as usize))
        };
        if result {
            self.program_brk = new_break as usize;
            Some(old_break)
        } else {
            None
        }
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
}