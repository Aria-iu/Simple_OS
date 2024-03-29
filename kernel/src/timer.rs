use riscv::register::time;
use crate::sbi::set_timer;
use crate::config::CLOCK_FREQ;


const TICKS_PER_SEC: usize = 100;
const MICRO_PER_SEC: usize = 1_000_000;

pub fn get_time() -> usize{
    time::read()
}

pub fn get_time_us() -> usize{
    let ts = time::read();
    ts / (CLOCK_FREQ/MICRO_PER_SEC)
}

pub fn set_next_trigger(){
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}