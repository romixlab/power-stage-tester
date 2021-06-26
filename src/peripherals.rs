use stm32f4xx_hal as hal;
use hal::{
    prelude::*,
    gpio::{
        gpioa::*, gpiob::*, gpioc::*, gpiod::PD2,
        Output, Input, Analog, Floating, Alternate, PushPull,
        AF1,
    }
};
use rtt_target::rprintln;

type OPP = Output<PushPull>;

pub struct BoardPeripherals {
    // pub rcc: hal::rcc::Rcc,
    pub clocks: hal::rcc::Clocks,
    pub delay: hal::delay::Delay,
    pub rtt_down_channel: rtt_target::DownChannel,
    pub adc: hal::adc::Adc<hal::pac::ADC1>,

    pub drv: Drv,
    pub switches: Option<Switches>,
    pub openloop: Option<OpenLoop>,
    pub feedback: Feedback,
    pub hall_sensors: HallSensors,
    pub canbus: CanBus,
    pub leds: Leds,
}

pub struct Drv {
    pub enable: PB5<OPP>,
    pub offset_cal: PB1<OPP>,
    pub fault: PB4<Input<Floating>>,
    pub spi: bitbang_hal::spi::SPI<
        PC11<Input<Floating>>,
        PC12<OPP>,
        PC10<OPP>,
        hal::timer::Timer<hal::pac::TIM6>
    >,
    pub cs: PD2<OPP>,
}

pub struct Switches {
    pub ah: PA8<OPP>,
    pub al: PB13<OPP>,
    pub bh: PA9<OPP>,
    pub bl: PB14<OPP>,
    pub ch: PA10<OPP>,
    pub cl: PB15<OPP>,
}

pub struct OpenLoop {
    pub ah: PA8<Alternate<AF1>>,
    pub al: PB13<Alternate<AF1>>,
    pub bh: PA9<Alternate<AF1>>,
    pub bl: PB14<Alternate<AF1>>,
    pub ch: PA10<Alternate<AF1>>,
    pub cl: PB15<Alternate<AF1>>,
}
impl OpenLoop {
    pub fn init(switches: Switches) -> Self {

        OpenLoop {
            ah: switches.ah.into_alternate_af1(),
            al: switches.al.into_alternate_af1(),
            bh: switches.bh.into_alternate_af1(),
            bl: switches.bl.into_alternate_af1(),
            ch: switches.ch.into_alternate_af1(),
            cl: switches.cl.into_alternate_af1()
        }
    }
    pub fn deinit(self) -> Switches {
        rprintln!("OpenLoop:deinit");
        Switches {
            ah: self.ah.into_push_pull_output(),
            al: self.al.into_push_pull_output(),
            bh: self.bh.into_push_pull_output(),
            bl: self.bl.into_push_pull_output(),
            ch: self.ch.into_push_pull_output(),
            cl: self.cl.into_push_pull_output()
        }
    }
}

pub struct Feedback {
    pub v_a: PA0<Analog>,
    pub v_b: PA1<Analog>,
    pub v_c: PA2<Analog>,
    pub i_a: PC2<Analog>,
    pub i_b: PC1<Analog>,
    pub i_c: PC0<Analog>,
    pub v_in: PC3<Analog>,
    pub temp_fet: PA3<Analog>,
    pub temp_motor: PC4<Analog>,
}

pub struct HallSensors {
    pub a: PC13<Input<Floating>>,
    pub b: PC14<Input<Floating>>,
    pub c: PC15<Input<Floating>>,
}
impl HallSensors {
    pub fn read(&self) -> (bool, bool, bool, u8) {
        let a = self.a.is_high().unwrap();
        let b = self.b.is_high().unwrap();
        let c = self.c.is_high().unwrap();
        let idx = ((a as u8) << 2) | ((b as u8) << 1) | (c as u8);
        (a, b, c, idx)
    }
}

pub struct CanBus {
    pub power_inject_enable: PA15<OPP>,
    pub voltage: PC5<Analog>,
    pub charge_pump_pwm: PB7<OPP>,
    pub standby_enable: PB3<OPP>,
}

pub struct Leds {
    pub red: PB2<OPP>,
    pub green: PB0<OPP>,
    pub blue: PC6<OPP>,
}