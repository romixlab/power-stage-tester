use crate::hal as hal;
use crate::peripherals::*;
use crate::hal::{
    gpio::{gpioa, gpiob, gpioc, gpiod, Output, PushPull},
    prelude::*,
    stm32::{interrupt, Interrupt, Peripherals, TIM2},
    timer::{Event, Timer},
    delay::Delay,
    adc::config::AdcConfig,
};

pub fn init_all() -> BoardPeripherals {
    let channels = rtt_target::rtt_init! {
        up: {
            0: { // channel number
                size: 1024 // buffer size in bytes
                mode: NoBlockSkip // mode (optional, default: NoBlockSkip, see enum ChannelMode)
                name: "Terminal" // name (optional, default: no name)
            }
        }
        down: {
            0: {
                size: 16
                name: "Terminal"
            }
        }
    };
    rtt_target::set_print_channel(channels.up.0);

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(8.mhz()).use_hse(8.mhz()).pclk1(8.mhz()).freeze();
    let delay = Delay::new(cp.SYST, clocks);

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    let gpiod = dp.GPIOD.split();

    let adc = hal::adc::Adc::adc1(dp.ADC1, true, AdcConfig::default());

    let drv_sck = gpioc.pc10.into_push_pull_output();
    let drv_miso = gpioc.pc11.into_floating_input();
    let drv_mosi = gpioc.pc12.into_push_pull_output();
    let spi_timer = hal::timer::Timer::tim6(dp.TIM6, 200.khz(), clocks);
    let spi = bitbang_hal::spi::SPI::new(
        bitbang_hal::spi::MODE_0,
        drv_miso,
        drv_mosi,
        drv_sck,
        spi_timer
    );

    // let can1_rx = gpiob.pb8;
    // let can1_tx = gpiob.pb9;
    // let can2_rx = gpiob.pb12;
    // let can2_tx = gpiob.pb6;

    BoardPeripherals {
        // rcc,
        clocks,
        delay,
        adc,
        rtt_down_channel: channels.down.0,
        drv: Drv {
            enable: gpiob.pb5.into_push_pull_output(),
            offset_cal: gpiob.pb1.into_push_pull_output(),
            fault: gpiob.pb4.into_floating_input(),
            spi,
            cs: gpiod.pd2.into_push_pull_output()
        },
        switches: Switches {
            ah: gpioa.pa8.into_push_pull_output(),
            al: gpiob.pb13.into_push_pull_output(),
            bh: gpioa.pa9.into_push_pull_output(),
            bl: gpiob.pb14.into_push_pull_output(),
            ch: gpioa.pa10.into_push_pull_output(),
            cl: gpiob.pb15.into_push_pull_output()
        },
        feedback: Feedback {
            v_a: gpioa.pa0.into_analog(),
            v_b: gpioa.pa1.into_analog(),
            v_c: gpioa.pa2.into_analog(),
            i_a: gpioc.pc2.into_analog(),
            i_b: gpioc.pc1.into_analog(),
            i_c: gpioc.pc0.into_analog(),
            v_in: gpioc.pc3.into_analog(),
            temp_fet: gpioa.pa3.into_analog(),
            temp_motor: gpioc.pc4.into_analog()
        },
        hall_sensors: HallSensors {
            a: gpioc.pc13.into_floating_input(),
            b: gpioc.pc14.into_floating_input(),
            c: gpioc.pc15.into_floating_input()
        },
        canbus: CanBus {
            power_inject_enable: gpioa.pa15.into_push_pull_output(),
            voltage: gpioc.pc5.into_analog(),
            charge_pump_pwm: gpiob.pb7.into_push_pull_output(),
            standby_enable: gpiob.pb3.into_push_pull_output()
        },
        leds: Leds {
            red: gpiob.pb2.into_push_pull_output(),
            green: gpiob.pb0.into_push_pull_output(),
            blue: gpioc.pc6.into_push_pull_output()
        }
    }
}