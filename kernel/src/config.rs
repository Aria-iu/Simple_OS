pub const MAX_APP_NUM:usize = 16;
// 0x80000000 - 0x80800000 8MB内存
// QEMU上可以设置更大内存范围
pub const MEMORY_END: usize = 0x80800000;


pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

#[cfg(feature="qemu")]
pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
];
#[cfg(feature="qemu")]
pub const CLOCK_FREQ: usize = 12500000;
