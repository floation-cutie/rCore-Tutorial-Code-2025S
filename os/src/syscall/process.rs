//! Process management syscalls

use crate::{
    config::PAGE_SIZE,
    mm::{translated_ptr, PageTable, PageTableEntry, PhysAddr, VirtAddr},
    task::{
        change_program_brk, count_syscall_times, current_user_token, exit_current_and_run_next, syscall_map, syscall_unmap, suspend_current_and_run_next
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
        },
        2 => count_syscall_times(id) as isize,
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
    syscall_map(start, len, port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if start % PAGE_SIZE != 0  {
        return -1;
    }
    syscall_unmap(start, len)
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
