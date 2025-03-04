# Lab1 实验报告

## 功能介绍
实现了系统调用 `sys_trace(trace_request: usize, id: usize, data: usize) -> isize`，支持三种操作：
1. trace_request = 0: 读取地址id处的一个字节
2. trace_request = 1: 写入一个字节data到地址id
3. trace_request = 2: 获取系统调用id的调用次数

## 实现过程
在TaskControlBlock中添加syscall_times数组记录系统调用计数。在syscall入口处统一计数，通过TaskManager提供update_syscall_times和get_current_syscall_times方法安全访问计数器。对于内存读写操作，使用unsafe代码块直接操作指针完成。整个实现通过封装私有字段并提供安全接口的方式，确保了数据访问的安全性。
