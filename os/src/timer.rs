//! RISC-V timer-related functionality

use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
use crate::sync::UPSafeCell;
use lazy_static::*;
use riscv::register::time;
/// The number of ticks per second
const TICKS_PER_SEC: usize = 100;
/// The number of milliseconds per second
const MSEC_PER_SEC: usize = 1000;
/// The number of microseconds per second
#[allow(dead_code)]
const MICRO_PER_SEC: usize = 1_000_000;

lazy_static! {
    static ref TIME_MS: UPSafeCell<usize> = unsafe { UPSafeCell::new(0) };
}

/// Get the current time in ticks
pub fn get_time() -> usize {
    time::read()
}

/// get current time in milliseconds
#[allow(dead_code)]
pub fn get_time_ms() -> usize {
    let mut time_ms = TIME_MS.exclusive_access();
    *time_ms += MSEC_PER_SEC / MSEC_PER_SEC;
    *time_ms
}

/// get current time in microseconds
#[allow(dead_code)]
pub fn get_time_us() -> usize {
    get_time_ms() * (MICRO_PER_SEC / MSEC_PER_SEC)
}

/// Set the next timer interrupt
pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
