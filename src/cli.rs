use crate::peripherals::BoardPeripherals;
use no_std_compat::prelude::v1::*;
use rtt_target::{rprint, rprintln};
use crate::vt100;
use embedded_hal::digital::v2::OutputPin;

type Args<'a> = &'a mut core::str::SplitAsciiWhitespace<'a>;

#[allow(unused_macros)]
macro_rules! ok_or_return {
    ($e: expr, $message: expr) => {
        match $e {
            Ok(x) => x,
            Err(_) => {
                rprintln!("{}Expected: {}{}", vt100::YELLOW, $message, vt100::DEFAULT);
                return;
            }
        }
    }
}

macro_rules! some_or_return {
    ($e: expr, $message: expr) => {
        match $e {
            Some(x) => x,
            None => {
                rprintln!("{}Expected: {}{}", vt100::YELLOW, $message, vt100::DEFAULT);
                return;
            }
        }
    }
}

macro_rules! unknown_command {
    ($message: expr) => {{
        rprintln!("{}Unknown command: {}{}", vt100::YELLOW, $message, vt100::DEFAULT);
        return;
    }}
}

macro_rules! command_executed {
    () => {
        rprintln!("{}Ok{}", vt100::GREEN, vt100::DEFAULT);
    }
}

pub fn process_input(bp: &mut BoardPeripherals) {
    let mut input = [0u8; 64];
    let input_len = bp.rtt_down_channel.read(&mut input);
    if input_len == 0 {
        return;
    }
    let input_str = core::str::from_utf8(&input[0..input_len]);
    match input_str {
        Ok(input_str) => {
            rprint!("{}: ", input_str);
            let mut args = input_str.split_ascii_whitespace();
            let cmd = args.next();
            match cmd {
                Some(cmd) => {
                    match cmd {
                        "drv" => {
                            drv_command(bp, &mut args);
                        }
                        "led" => {
                            led_command(bp, &mut args);
                        }
                        "sw" => {
                            switch_command(bp, &mut args);
                        }
                        "swmode" => {
                            switch_mode_command(bp, &mut args);
                        }
                        _ => {
                            rprintln!("Unknown command: {}", cmd);
                        }
                    }
                }
                None => {
                    rprintln!("Empty command");
                }
            }
        },
        Err(_) => {
            rprintln!("Non utf-8 command");
        }
    }
}

fn drv_command(bp: &mut BoardPeripherals, args: Args) {
    let cmd = some_or_return!(args.next(), "drv on/off/regs/gain/reset");
    match cmd {
        "on" => {
            bp.drv.enable.set_high().ok();
        }
        "off" => {
            bp.drv.enable.set_low().ok();
        }
        "regs" => {

        }
        "gain" => {

        }
        "reset" => {

        }
        _ => unknown_command!(cmd)
    }
    command_executed!()
}

fn switch_command(bp: &mut BoardPeripherals, args: Args) {
    let switches = match &mut bp.switches {
        Some(s) => s,
        None => {
            rprintln!("Manual control disabled");
            return;
        }
    };
    let cmd = some_or_return!(args.next(), "sw ah/al/az bh/bl/bz ch/cl/cz");
    match cmd {
        "ah" => {
            switches.ah.set_high().ok();
            switches.al.set_low().ok();
        }
        "al" => {
            switches.ah.set_low().ok();
            switches.al.set_high().ok();
        }
        "az" => {
            switches.ah.set_low().ok();
            switches.al.set_low().ok();
        }

        "bh" => {
            switches.bh.set_high().ok();
            switches.bl.set_low().ok();
        }
        "bl" => {
            switches.bh.set_low().ok();
            switches.bl.set_high().ok();
        }
        "bz" => {
            switches.bh.set_low().ok();
            switches.bl.set_low().ok();
        }

        "ch" => {
            switches.ch.set_high().ok();
            switches.cl.set_low().ok();
        }
        "cl" => {
            switches.ch.set_low().ok();
            switches.cl.set_high().ok();
        }
        "cz" => {
            switches.ch.set_low().ok();
            switches.cl.set_low().ok();
        }
        _ => unknown_command!(cmd)
    }
    command_executed!()
}

fn switch_mode_command(bp: &mut BoardPeripherals, args: Args) {
    let cmd = some_or_return!(args.next(), "swmode manual/openloop");
    match cmd {
        "manual" => {
            match bp.switches {
                Some(_) => {
                    rprintln!("Already in manual");
                },
                None => {
                    rprintln!("Switching to manual");
                    let openloop = bp.openloop.take().unwrap();
                    let switches = openloop.deinit();
                    bp.switches = Some(switches)
                }
            }
        }
        "openloop" => {
            match bp.openloop {
                Some(_) => {
                    rprintln!("Already in openloop");
                }
                None => {
                    rprintln!("Switching to openloop");
                    let switches = bp.switches.take().unwrap();
                    let openloop = crate::peripherals::OpenLoop::init(switches);
                    bp.openloop = Some(openloop);
                }
            }
        }
        _ => {
            unknown_command!(cmd)
        }
    }
    command_executed!()
}

fn led_command(bp: &mut BoardPeripherals, args: Args) {
    let led = some_or_return!(args.next(), "led red/green/blue on/off");
    let cmd = some_or_return!(args.next(), "on/off");
    let is_on = match cmd {
        "on" => {
            true
        }
        "off" => {
            false
        }
        _ => unknown_command!(cmd)
    };
    match led {
        "red" => {
            if is_on {
                bp.leds.red.set_high().ok();
            } else {
                bp.leds.red.set_low().ok();
            }
        }
        "green" => {
            if is_on {
                bp.leds.green.set_high().ok();
            } else {
                bp.leds.green.set_low().ok();
            }
        }
        "blue" => {
            if is_on {
                bp.leds.blue.set_high().ok();
            } else {
                bp.leds.blue.set_low().ok();
            }
        }
        _ => unknown_command!(led)
    }
    command_executed!()
}