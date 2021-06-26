use crate::peripherals::{Switches};
use stm32f4xx_hal as hal;
use hal::{
    // prelude::*,
    gpio::{gpioa::*, gpiob::*, Alternate, AF1}
};
use rtt_target::rprintln;
use stm32f4xx_hal::time::Hertz;

const PWM_FREQ: Hertz = Hertz(20_000);

pub enum Phase {
    A,
    B,
    C
}

pub struct OpenLoop {
    pub ah: PA8<Alternate<AF1>>,
    pub al: PB13<Alternate<AF1>>,
    pub bh: PA9<Alternate<AF1>>,
    pub bl: PB14<Alternate<AF1>>,
    pub ch: PA10<Alternate<AF1>>,
    pub cl: PB15<Alternate<AF1>>,
    duty_a: u32,
    duty_b: u32,
    duty_c: u32
}
impl OpenLoop {
    pub fn init(core_freq: Hertz, switches: Switches) -> Self {
        init_tim1(core_freq, PWM_FREQ);

        let duty = Self::arr() / 2;
        OpenLoop {
            ah: switches.ah.into_alternate_af1(),
            al: switches.al.into_alternate_af1(),
            bh: switches.bh.into_alternate_af1(),
            bl: switches.bl.into_alternate_af1(),
            ch: switches.ch.into_alternate_af1(),
            cl: switches.cl.into_alternate_af1(),

            duty_a: duty,
            duty_b: duty,
            duty_c: duty,
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

    fn arr() -> u32 {
        let dp = unsafe {
            hal::pac::Peripherals::steal()
        };
        dp.TIM1.arr.read().bits()
    }

    pub fn update_duty(&mut self, phase: Phase, duty: u8) {
        let duty = if duty > 100 {
            100
        } else {
            duty
        };
        let duty = duty as u32 * Self::arr() / 100;
        match phase {
            Phase::A => {
                self.duty_a = duty;
            }
            Phase::B => {
                self.duty_b = duty;
            }
            Phase::C => {
                self.duty_c = duty;
            }
        }
        let dp = unsafe {
            hal::pac::Peripherals::steal()
        };
        dp.TIM1.cr1.modify(|_, w| w.udis().disabled());
        dp.TIM1.ccr1.write(|w| unsafe { w.bits(self.duty_a) });
        dp.TIM1.ccr2.write(|w| unsafe { w.bits(self.duty_b) });
        dp.TIM1.ccr3.write(|w| unsafe { w.bits(self.duty_c) });
        dp.TIM1.cr1.modify(|_, w| w.udis().enabled());
    }
}

fn init_tim1(core_freq: Hertz, pwm_freq: Hertz, ) {
    let dp = unsafe {
        hal::pac::Peripherals::steal()
    };
    dp.RCC.apb2enr.modify(|_, w| w.tim1en().enabled());
    dp.RCC.apb2rstr.modify(|_, w| w.tim1rst().set_bit());
    dp.RCC.apb2rstr.modify(|_, w| w.tim1rst().clear_bit());

    dp.TIM1.cr1.write(|w| w
        .cms().center_aligned1()
        .ckd().div1()
    );
    let arr_bits = (core_freq.0 / pwm_freq.0) as u16;
    dp.TIM1.arr.write(|w| w.arr().bits(arr_bits));
    dp.TIM1.psc.write(|w| w.psc().bits(0));
    dp.TIM1.rcr.write(|w| unsafe { w.rep().bits(0) });
    dp.TIM1.egr.write(|w| w.ug().update());

    // Disable output compare 1,2,3
    dp.TIM1.ccer.modify(|_, w| w
        .cc1e().clear_bit().cc1ne().clear_bit()
        .cc2e().clear_bit().cc2ne().clear_bit()
        .cc3e().clear_bit().cc3ne().clear_bit()
    );
    // Output idle and idle_n state
    dp.TIM1.cr2.modify(|_, w| w
        .ois1().set_bit().ois1n().set_bit()
        .ois2().set_bit().ois2n().set_bit()
        .ois3().set_bit().ois3n().set_bit()
    );
    // Select output mode
    dp.TIM1.ccmr1_output_mut().modify(|_, w| w
        .oc1m().pwm_mode1()
        .oc2m().pwm_mode1()
    );
    dp.TIM1.ccmr2_output_mut().modify(|_, w| w
        .oc3m().pwm_mode1()
    );
    dp.TIM1.ccr1.write(|w| w.ccr().bits(arr_bits / 2));
    dp.TIM1.ccr2.write(|w| w.ccr().bits(arr_bits / 2));
    dp.TIM1.ccr3.write(|w| w.ccr().bits(arr_bits / 2));
    dp.TIM1.ccer.modify(|_, w| w
        // polarity
        .cc1p().set_bit()
        .cc2p().set_bit()
        .cc3p().set_bit()
        // enable outputs
        .cc1e().set_bit().cc1ne().set_bit()
        .cc2e().set_bit().cc2ne().set_bit()
        .cc3e().set_bit().cc3ne().set_bit()
    );
    // Enable preload
    dp.TIM1.ccmr1_output_mut().modify(|_, w| w.oc1pe().enabled().oc2pe().enabled());
    dp.TIM1.ccmr2_output_mut().modify(|_, w| w.oc3pe().set_bit());
    // Dead time, break disable
    dp.TIM1.bdtr.write(|w| unsafe { w
        .ossr().idle_level()
        .ossi().idle_level()
        .lock().bits(0)
        .dtg().bits(127) // TODO: calculate proper dead time
        .aoe().clear_bit()
        .bke().clear_bit()
        .bkp().set_bit()
    });
    // Preload enable on CCR and ARR
    dp.TIM1.cr2.modify(|_, w| w.ccpc().set_bit());
    dp.TIM1.cr1.modify(|_, w| w.arpe().set_bit());
    // Enable
    // dp.TIM1.cnt.write(0)
    dp.TIM1.cr1.modify(|_, w| w.cen().enabled());
    dp.TIM1.bdtr.modify(|_, w| w.moe().enabled());
}

