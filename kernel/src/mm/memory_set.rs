use alloc::sync::Arc;
use alloc::{collections::BTreeMap, vec::Vec};
use bitflags::*;
use crate::config::MEMORY_END;
use crate::config::PAGE_SIZE;
use crate::config::TRAMPOLINE;
use crate::config::TRAP_CONTEXT;
use crate::config::USER_STACK_SIZE;
use crate::mm::address::StepByOne;
use crate::config::MMIO;
use crate::sync::UPSafeCell;
use super::frame_alloc;
use super::page_table::PTEFlags;
use super::PageTableEntry;
use super::PhysPageNum;
use super::VirtAddr;
use super::PageTable;
use super::{address::VirtPageNum, FrameTracker, VPNRange};
use super::PhysAddr;
use riscv::register::satp;
use core::arch::asm;
// 逻辑段
pub struct MapArea{
    //  VPNRange 描述一段虚拟页号的连续区间，
    //  表示该逻辑段在地址区间中的位置和长度
    vpn_range: VPNRange,
    // 当逻辑段采用 MapType::Framed 方式映射到物理内存的时候，
    // data_frames 是一个保存了该逻辑段内的每个虚拟页面和
    // 它被映射到的物理页帧 FrameTracker 的一个键值对容器 BTreeMap 中，
    // 这些物理页帧被用来存放实际内存数据而不是作为多级页表中的中间节点。
    data_frames: BTreeMap<VirtPageNum,FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}


// 恒等映射方式主要是用在启用多级页表之后，
// 内核仍能够在虚存地址空间中访问一个特定的物理地址指向的物理内存
#[derive(Clone, Copy,PartialEq,Debug)]
pub enum MapType{
    // Identical 表示恒等映射方式
    Identical,
    // Framed 则表示对于每个虚拟页面都有一个新分配的
    // 物理页帧与之对应，虚地址与物理地址的映射关系是相对随机的
    Framed,
}

bitflags!{
    pub struct MapPermission: u8{
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

impl MapArea{
    pub fn new(start_va: VirtAddr,end_va: VirtAddr,map_type: MapType,map_perm: MapPermission) -> Self{
        let start_vpn: VirtPageNum = start_va.floor();
        let end_vpn = end_va.ceil();
        Self{
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }
    // 将vpn映射到ppn上，并存储到page_table中，
    // 若是Framed映射，在当前逻辑段的data_frames中插入信息
    pub fn map_one(&mut self,page_table: &mut PageTable, vpn: VirtPageNum){
        let ppn: PhysPageNum;
        match self.map_type{
            MapType::Identical => {
                ppn = PhysPageNum(vpn.0);
            },
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
            },
        }
        let pteflags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
        page_table.map(vpn, ppn, pteflags);
    }
    #[allow(unused)]
    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum){
        if self.map_type == MapType::Framed {
            self.data_frames.remove(&vpn);
        }
        page_table.unmap(vpn);
    }
    // 将VPNRange中的虚拟页号全部映射到物理页号上，不论映射方式
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }
    #[allow(unused)]
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }
    // 缩减当前逻辑段的长度
    #[allow(unused)]
    pub fn shrink_to(&mut self, page_table: &mut PageTable, new_end: VirtPageNum) {
        for vpn in VPNRange::new(new_end, self.vpn_range.get_end()) {
            self.unmap_one(page_table, vpn)
        }
        self.vpn_range = VPNRange::new(self.vpn_range.get_start(), new_end);
    }
    // 增加当前逻辑段的长度
    #[allow(unused)]
    pub fn append_to(&mut self, page_table: &mut PageTable, new_end: VirtPageNum) {
        for vpn in VPNRange::new(self.vpn_range.get_end(), new_end) {
            self.map_one(page_table, vpn)
        }
        self.vpn_range = VPNRange::new(self.vpn_range.get_start(), new_end);
    }
    // 将数据data拷贝到当前逻辑段
    pub fn copy_data(&mut self,page_table: &mut PageTable,data:&[u8]){
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.get_start();
        let len = data.len();
        loop{
            let src = &data[start..len.min(start + PAGE_SIZE)];
            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len{
                break;
            }
            current_vpn.step();
        }
    }
}


extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}


// 地址空间：一系列有关联的逻辑段
pub struct MemorySet{
    page_table: PageTable,
    areas: Vec<MapArea>,
}

lazy_static! {
    /// a memory set instance through lazy_static! managing kernel space
    pub static ref KERNEL_SPACE: Arc<UPSafeCell<MemorySet>> =
        Arc::new(unsafe { UPSafeCell::new(MemorySet::new_kernel()) });
}

impl MemorySet {
    pub fn new_bare() -> Self{
        Self{
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }
    // 1...root.ppn(64位)
    pub fn token(&self) -> usize{
        self.page_table.token()
    }

    fn push(&mut self,mut map_area:MapArea,data: Option<&[u8]>){
        map_area.map(&mut self.page_table);
        if let Some(data) = data{
            map_area.copy_data(&mut self.page_table, data);
        }
        self.areas.push(map_area);
    }

    pub fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ){
        self.push(MapArea::new(start_va, end_va, MapType::Framed, permission), None);
    }

    /// Mention that trampoline is not collected by areas.
    /// trampoline没有被areas收集为MapArea
    fn map_trampoline(&mut self) {
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }

    // 生成内核地址空间
    pub fn new_kernel() -> Self{
        let mut memory_set = Self::new_bare();
        memory_set.map_trampoline();
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(
            ".bss [{:#x}, {:#x})",
            sbss_with_stack as usize, ebss as usize
        );
        println!("mapping .text section");
        memory_set.push(
            MapArea::new(
                (stext as usize).into(),
                (etext as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );
        println!("mapping .rodata section");
        memory_set.push(
            MapArea::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                MapType::Identical,
                MapPermission::R,
            ),
            None,
        );
        println!("mapping .data section");
        memory_set.push(
            MapArea::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping .bss section");
        memory_set.push(
            MapArea::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping physical memory");
        memory_set.push(
            MapArea::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        println!("mapping memory-mapped registers");
        for pair in MMIO {
            memory_set.push(
                MapArea::new(
                    (*pair).0.into(),
                    ((*pair).0 + (*pair).1).into(),
                    MapType::Identical,
                    MapPermission::R | MapPermission::W,
                ),
                None,
            );
        }
        memory_set
    }

    // 从elf文件中生成用户程序地址空间
    // Include sections in elf and trampoline 
    // and TrapContext and user stack,
    // also returns user_sp and entry point.
    pub fn from_elf(elf_data:&[u8]) -> (Self,usize,usize){
        let mut memory_set = Self::new_bare();
        memory_set.map_trampoline();
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        for i in 0..ph_count{
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load{
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() {
                    map_perm |= MapPermission::W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MapPermission::X;
                }
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
                max_end_vpn = map_area.vpn_range.get_end();
                memory_set.push(
                    map_area,
                    Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]),
                );
            }
        }
        let max_end_va: VirtAddr = max_end_vpn.into();
        let mut user_stack_bottom: usize = max_end_va.into();
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(
            MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );
        memory_set.push(
            MapArea::new(
                user_stack_top.into(),
                user_stack_top.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );
        memory_set.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        (
            memory_set,
            user_stack_top,
            elf.header.pt2.entry_point() as usize,
        )
    }

    pub fn acticate(&self){
        // 设置根页表
        let satp = self.page_table.token();
        unsafe{
            satp::write(satp);
            // 刷新指令缓存
            asm!("sfence.vma");
        }
    }

    pub fn translate(&mut self,vpn: VirtPageNum) -> Option<PageTableEntry>{
        self.page_table.translate(vpn)
    }

    #[allow(unused)]
    pub fn shrink_to(&mut self,start: VirtAddr,new_end: VirtAddr)->bool{
        if let Some(area) = self.areas
            .iter_mut()
            .find(|area|{area.vpn_range.get_start()==start.floor()})
        {
            area.shrink_to(&mut self.page_table, new_end.ceil());
            true
        }else{
            false
        }
    }
    #[allow(unused)]
    pub fn append_to(&mut self,start: VirtAddr,new_end: VirtAddr)->bool{
        if let Some(area) = self.areas.iter_mut().
        find(|area|area.vpn_range.get_start()==start.floor()) {
            area.append_to(&mut self.page_table, new_end.ceil());
            true
        }else{
            false
        }
    }
}

#[allow(unused)]
pub fn remap_test() {
    let mut kernel_space = KERNEL_SPACE.exclusive_access();
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();
    assert!(!kernel_space
        .page_table
        .translate(mid_text.floor())
        .unwrap()
        .writable(),);
    assert!(!kernel_space
        .page_table
        .translate(mid_rodata.floor())
        .unwrap()
        .writable(),);
    assert!(!kernel_space
        .page_table
        .translate(mid_data.floor())
        .unwrap()
        .executable(),);
    println!("remap_test passed!");
}
