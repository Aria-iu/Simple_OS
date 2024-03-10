use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
pub struct TrapContext{
    pub reg: [usize;32],
    pub sstatus: Sstatus,
    pub sepc: usize,
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

