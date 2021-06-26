#![no_main]
#![no_std]

mod peripherals;
mod init;
mod cli;
mod observer;
mod vt100;

use panic_rtt_target as _;
use embedded_hal::blocking::delay::DelayMs;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut bp = init::init_all();

    loop {
        observer::print_system_status(&mut bp);
        cli::process_input(&mut bp);
        bp.delay.delay_ms(50_u32);
    }
}

use cortex_m_rt::exception;
#[exception]
fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    panic!("HF: {:#?}", ef);
}