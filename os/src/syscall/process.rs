//! Process management syscalls
use crate::{
    config::PAGE_SIZE,
    mm::{translated_byte_buffer, MapPermission, PTEFlags, VirtAddr},
    task::{
        change_program_brk, current_syscall_times, current_translate, current_user_token,
        exit_current_and_run_next, mmap_current, munmap_current, suspend_current_and_run_next,
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

/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let timeval = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let src = unsafe {
        core::slice::from_raw_parts(
            &timeval as *const TimeVal as *const u8,
            core::mem::size_of::<TimeVal>(),
        )
    };
    let mut start = 0;
    for dst in translated_byte_buffer(current_user_token(), ts as *const u8, src.len()) {
        let end = start + dst.len();
        dst.copy_from_slice(&src[start..end]);
        start = end;
    }
    0
}

/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    let va = VirtAddr::from(id);
    match trace_request {
        0 => {
            let pte = match current_translate(va) {
                Some(pte) if pte.flags().contains(PTEFlags::U) && pte.readable() => pte,
                _ => return -1,
            };
            pte.ppn().get_bytes_array()[va.page_offset()] as isize
        }
        1 => {
            let pte = match current_translate(va) {
                Some(pte) if pte.flags().contains(PTEFlags::U) && pte.writable() => pte,
                _ => return -1,
            };
            pte.ppn().get_bytes_array()[va.page_offset()] = data as u8;
            0
        }
        2 => current_syscall_times(id) as isize,
        _ => -1,
    }
}

pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if start % PAGE_SIZE != 0 || len == 0 || port == 0 || port & !0x7 != 0 {
        return -1;
    }
    if port & 0x2 != 0 && port & 0x1 == 0 {
        return -1;
    }
    let mut permission = MapPermission::U;
    if port & 0x1 != 0 {
        permission |= MapPermission::R;
    }
    if port & 0x2 != 0 {
        permission |= MapPermission::W;
    }
    if port & 0x4 != 0 {
        permission |= MapPermission::X;
    }
    if mmap_current(start, len, permission) {
        0
    } else {
        -1
    }
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if start % PAGE_SIZE != 0 || len == 0 {
        return -1;
    }
    if munmap_current(start, len) {
        0
    } else {
        -1
    }
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
