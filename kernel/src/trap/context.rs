use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
pub struct TrapContext{
    pub reg: [usize;32],
    pub sstatus: Sstatus,
    pub sepc: usize,
    //  kernel_satp 表示内核地址空间的 token ，即内核页表的起始物理地址；
    //  kernel_sp 表示当前应用在内核地址空间中的内核栈栈顶的虚拟地址；
    //  trap_handler 表示内核中 trap handler 入口点的虚拟地址
    //  它们在应用初始化的时候由内核写入应用地址空间中的 
    //  TrapContext 的相应位置，此后就不再被修改
    pub kernel_satp: usize,
    pub kernel_sp: usize,
    pub trap_handler: usize,
}

impl TrapContext{
    fn set_sp(&mut self, sp: usize){
        self.reg[2] = sp;
    }

    // 一些便捷的操作上下文的函数...
    // ...

    pub fn app_init_context(addr: usize, sp: usize) -> Self{
        let mut sstatus = sstatus::read(); // CSR sstatus
        sstatus.set_spp(SPP::User); //previous privilege mode: user mode
        let mut cx = Self {
            reg: [0; 32],
            sstatus,
            sepc: addr, // entry point of app
        };
        cx.set_sp(sp); // app's user stack pointer
        cx // return initial Trap Context of app
    }
}

