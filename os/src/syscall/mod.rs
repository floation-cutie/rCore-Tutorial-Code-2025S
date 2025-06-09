//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.
const SYSCALL_WRITE: usize = 64;
/// exit syscall
const SYSCALL_EXIT: usize = 93;
/// yield syscall
const SYSCALL_YIELD: usize = 124;
/// gettime syscall
const SYSCALL_GET_TIME: usize = 169;
/// sbrk syscall
const SYSCALL_SBRK: usize = 214;
/// munmap syscall
const SYSCALL_MUNMAP: usize = 215;
/// mmap syscall
const SYSCALL_MMAP: usize = 222;
/// trace syscall
const SYSCALL_TRACE: usize = 410;

mod fs;
mod process;

use fs::*;
use process::*;
use crate::task::TASK_MANAGER;

/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    TASK_MANAGER.update_syscall_times(syscall_id);
    
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TRACE => sys_trace(args[0], args[1], args[2]),
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2]),
        SYSCALL_MUNMAP => sys_munmap(args[0], args[1]),
        SYSCALL_SBRK => sys_sbrk(args[0] as i32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}

use crate::trap::TrapContext;

/// syscall id array
pub const SYSCALL_CMD_ARRAY: [usize; 8] = [
    SYSCALL_WRITE,
    SYSCALL_EXIT,
    SYSCALL_YIELD,
    SYSCALL_GET_TIME,
    SYSCALL_TRACE,
    SYSCALL_MMAP,
    SYSCALL_MUNMAP,
    SYSCALL_SBRK,
];
/// get index of syscall
fn index_of_syscall(syscall_id: usize) -> Option<usize> {
    SYSCALL_CMD_ARRAY.iter().position(|&x| x == syscall_id)
}

/// trace syscall
pub fn trace_syscall(cx : &mut TrapContext) {
    match index_of_syscall(cx.x[17]) {
        Some(index)=> {
            unsafe {
                // 获取到命令的调用计数指针地址并+1
                let mut next_ptr = (cx as *mut TrapContext).offset(1) as *mut usize;
                next_ptr = next_ptr.offset(index as isize);
                *next_ptr = *next_ptr + 1;
                // if index != 2 && index != 3 {
                //     println!("[kernel] syscall {} index: {} count:{}.", cx.x[17], index, *next_ptr);
                // }
                // 追踪syscall调用，x[10]=2表示查询次数
                if cx.x[17] == SYSCALL_TRACE && cx.x[10] == 2 {
                    // 读取查询id的调用次数
                    match index_of_syscall(cx.x[11]) {
                        Some(cmd_index) => {
                            // 只要偏移cmd_index - index，即是对应id的调用次数
                            let cmd_count_ptr = next_ptr.offset((cmd_index - index) as isize);
                            // 写入到data，由sys_trace()读取返回
                            cx.x[12] = *cmd_count_ptr;
                        }
                        _ => panic!("Unsupported syscall_id: {}", cx.x[11]),
                    }

                }
            }
        }
        _ => panic!("Unsupported syscall_id: {}", cx.x[17]),
    }

}