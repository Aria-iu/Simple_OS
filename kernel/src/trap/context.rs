use riscv::register::sstatus::{self,Sstatus,SSP};

#[repr(C)]
pub struct TrapContext{
    pub reg: [usize;32],
    pub sstatus: Sstatus,
    pub sepc: usize,
}