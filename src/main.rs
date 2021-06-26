#![no_main]
#![no_std]

mod peripherals;
mod init;
mod cli;
mod observer;
mod vt100;

use panic_rtt_target as _;
use stm32f4xx_hal as hal;
use cortex_m_rt::entry;
use rtt_target::rprintln;
use embedded_hal::blocking::delay::DelayMs;

#[entry]
fn main() -> ! {
    let mut bp = init::init_all();
    rprintln!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

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