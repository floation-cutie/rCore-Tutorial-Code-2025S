//! Process management syscalls

use crate::{
    config::PAGE_SIZE,
    mm::{frame_alloc, translated_ptr, PTEFlags, PageTable, PageTableEntry, PhysAddr, StepByOne, VirtAddr},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        let addr = translated_ptr(current_user_token(), ts as usize);
        let ts_mut = addr as *mut TimeVal;
        (*ts_mut).sec = us / 1_000_000;
        (*ts_mut).usec = us % 1_000_000;
    }
    0
}

fn get_addr_pte(ptr: usize) -> Option<PageTableEntry> {
    let page_table = PageTable::from_token(current_user_token());
    let ptr_va = VirtAddr::from(ptr);
    let vpn = ptr_va.floor();
    page_table.translate(vpn)
}
/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            if let Some(pte) = get_addr_pte(id) {
                if pte.is_valid() && pte.readable() && pte.is_user()  {
                    let va = VirtAddr::from(id);
                    let ptr = PhysAddr::from(pte.ppn()).0 + va.page_offset();
                    let ret = ptr as *const u8;
                    unsafe {
                        return *ret as isize;
                    }
                }
            }
            -1
        },
        1 => {
            if let Some(pte) = get_addr_pte(id) {
                if pte.is_valid() && pte.writable() && pte.is_user()  {
                    let va = VirtAddr::from(id);
                    let ptr = PhysAddr::from(pte.ppn()).0 + va.page_offset();
                    unsafe {
                        *(ptr as *mut u8) = data as u8;
                    }
                    return 0;
                }
            }
            -1
        }
        2 => {
            return data as isize;
        }
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if port & !0x7 != 0 || port & 0x7 == 0 {
        return -1;
    }
    if start % PAGE_SIZE != 0  {
        return -1;
    }
    let mut page_table = PageTable::from_token(current_user_token());
    let ptr_va = VirtAddr::from(start);
    let mut vpn = ptr_va.floor();
    let page_count = (len + PAGE_SIZE - 1) / PAGE_SIZE;
    println!("sys_mmap: start = 0x{:x}, len = {}, page_count = {}", start, len, page_count);
    for i in 0..page_count {
        println!("sys_mmap: i = {}, vpn=0x{:x}", i, vpn.0);
        // 只要有一个page被映射了，就返回失败
        let pte = page_table.translate(vpn);
        match pte {
            None => {
                // continue
            }
            _ => {
                if pte.unwrap().is_valid() {
                    // println!("sys_mmap: pte is {:x}", pte.unwrap().bits);
                    return -1;
                }
            }
        }
        
        vpn.step();
    }
    // 重新指向起始page
    let mut vpn =  VirtAddr::from(start).floor();
    let mut flags = PTEFlags::U | PTEFlags::V;
    if port & 0x1 != 0 {
        flags |= PTEFlags::R;
    }
    if port & 0x2 != 0 {
        flags |= PTEFlags::W;
    }
    if port & 0x4 != 0 {
        flags |= PTEFlags::X;
    }
    for _ in 0..page_count {
        let frame = frame_alloc().unwrap();
        let ppn = frame.ppn;
        page_table.map(vpn, ppn, flags);
        vpn.step();
    }
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if start % PAGE_SIZE != 0  {
        return -1;
    }
    let mut page_table = PageTable::from_token(current_user_token());
    let ptr_va = VirtAddr::from(start);
    let mut vpn = ptr_va.floor();
    let page_count = (len + PAGE_SIZE - 1) / PAGE_SIZE;
    println!("sys_munmap: start = 0x{:x}, len = {}, page_count = {}", start, len, page_count);
    for _ in 0..page_count {
        // 只要有一个page未被映射，就返回失败
        let pte = page_table.translate(vpn);
        match pte {
            None => {
                return -1;
            }
            _ => {
                if !pte.unwrap().is_valid() {
                    println!("sys_munmap: vpn is {:x} pte is {:x}", vpn.0, pte.unwrap().bits);
                    return -1;
                }
            }
        }
        vpn.step();
    }
    let mut vpn = VirtAddr::from(start).floor();
    for _ in 0..page_count {
        page_table.unmap(vpn);
        vpn.step();
    }
    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
