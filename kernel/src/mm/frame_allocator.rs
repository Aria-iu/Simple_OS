use core::fmt;
use core::fmt::Formatter;
use core::fmt::Debug;

use alloc::vec::Vec;

use crate::config::MEMORY_END;
use crate::sync::UPSafeCell;

use super::{PhysAddr, PhysPageNum};

pub struct FrameTracker{
    pub ppn: PhysPageNum,
}

impl FrameTracker{
    pub fn new(ppn: PhysPageNum)-> Self{
        let bytes_mut = ppn.get_bytes_array();
        for i in bytes_mut{
            *i = 0;
        }
        Self{
            ppn,
        }
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

pub struct StackFrameAllocator{
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl StackFrameAllocator{
    pub fn init(&mut self,l: PhysPageNum,r: PhysPageNum){
        self.current = l.0;
        self.end = r.0;
    }
}

impl FrameAllocator for StackFrameAllocator{
    fn new() -> Self {
        Self{
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop(){
            Some(ppn.into())
        }else if self.current == self.end {
            None
        }else {
            self.current += 1;
            Some((self.current - 1).into())
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        if ppn >= self.current || self.recycled.iter().any(|&v| v==ppn){
            panic!("Frame ppn={:#x} has not been allocated!", ppn)
        }else {
            self.recycled.push(ppn);
        }
    }
}

// rename it
type FrameAllocatorImpl = StackFrameAllocator;

lazy_static! {
    /// frame allocator instance through lazy_static!
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> =
        unsafe { UPSafeCell::new(FrameAllocatorImpl::new()) };
}

pub fn init_frame_allocator(){
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil(), 
            PhysAddr::from(MEMORY_END).floor()
        );
}

pub fn frame_alloc() -> Option<FrameTracker>{
    FRAME_ALLOCATOR.exclusive_access().alloc().map(FrameTracker::new)
}

pub fn frame_dealloc(ppn : PhysPageNum){
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn)
}

#[allow(unused)]
/// a simple test for frame allocator
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}

