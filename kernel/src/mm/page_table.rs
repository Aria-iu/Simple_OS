/// 页表数据结构
/// 如果使用线性表存储PTE表项，那么虚拟页号有2^(39-12)种，
/// 每个虚拟页号对应一个表项，每个页表需要2^30 = 1GB的内存。
/// 应用的数据还需要保存在内存的其他位置，
/// 这就使得每个应用要吃掉 1GiB 以上的内存。
/// 按需分配，也就是说：有多少合法的虚拟页号，我们就维护一个多大
/// 的映射，并为此使用多大的内存用来保存映射。
/// 使用多级页表，SV39的虚拟页号被分为三级页索引
/// 

use bitflags::*;
use super::address::VirtPageNum;
use super::{frame_alloc, FrameTracker, PhysPageNum};
use alloc::vec::Vec;
use alloc::vec;

bitflags! {
    /// page table entry flags
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry{
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize,
        }
    }
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }
}

/// page table structure
pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

impl PageTable{
    pub fn new() -> Self {
        let frame: FrameTracker = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }
    pub fn from_token(satp: usize)->Self{
        Self{
            root_ppn: PhysPageNum::from(satp&((1usize<<44)-1)),
            frames: Vec::new(),
        }
    }
    fn find_pte_create(&mut self,vpn: VirtPageNum) -> Option<&mut PageTableEntry>{
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i,idx) in idxs.iter().enumerate(){
            let pte = &mut ppn.get_pte_array()[*idx];
            if i==2{
                result = Some(pte);
                break;
            }
            if !pte.is_valid(){
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }
    fn find_pte(&self,vpn: VirtPageNum) -> Option<&mut PageTableEntry>{
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i,idx) in idxs.iter().enumerate(){
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }

}
