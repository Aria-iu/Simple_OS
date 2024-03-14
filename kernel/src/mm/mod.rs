mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use page_table::PageTableEntry;
pub use address::PhysAddr;
pub use address::PhysPageNum;
pub use frame_allocator::FrameTracker;
pub use frame_allocator::frame_alloc;
pub use address::VPNRange;
pub use page_table::PageTable;
pub use address::VirtAddr;
pub use memory_set::MemorySet;
pub use memory_set::MapPermission;
pub use page_table::translate_byte_buffer;
pub use memory_set::remap_test;

pub use memory_set::KERNEL_SPACE;
pub fn init(){
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().acticate();
}