use crate::peripherals::BoardPeripherals;
use rtt_target::{rprintln, rprint};
use crate::vt100;
use stm32f4xx_hal::adc::config::SampleTime;
use embedded_hal::digital::v2::InputPin;

const ADC_SAMPLE_TIME: SampleTime = SampleTime::Cycles_28;
const RT: Ohms = Ohms(34900);
const RB: Ohms = Ohms(4990);

const SHUNT: MicroOhms = MicroOhms(10_000);
const ADC_I_MIDPOINT: MilliVolts = MilliVolts(1650);

macro_rules! print_phase_voltage {
    ($bp: expr, $an_pin: ident) => {
        let sample = $bp.adc.convert(&mut $bp.feedback.$an_pin, ADC_SAMPLE_TIME);
        let v_adc = $bp.adc.sample_to_millivolts(sample);
        let v = resistor_divider_inverse(RT, RB, MilliVolts(v_adc as i32));
        rprint!(=>1, "Raw={}\tVadc={}mV\tV={}", sample, v_adc, v);
    }
}

macro_rules! print_phase_current {
    ($bp: expr, $an_pin: ident, $gain: expr) => {
        let sample = $bp.adc.convert(&mut $bp.feedback.$an_pin, ADC_SAMPLE_TIME);
        let v_adc = $bp.adc.sample_to_millivolts(sample);
        let i = voltage_to_current(MilliVolts(v_adc as i32), ADC_I_MIDPOINT, SHUNT, $gain);
        rprint!(=>1, "Raw={}\tVadc={}mV\tI={}", sample, v_adc, i);
    }
}

pub fn print_system_status(bp: &mut BoardPeripherals) {
    rprintln!(=>1, "{}", vt100::CLEAR_SCREEN);

    let is_drv_on = bp.drv.enable.is_high().unwrap();
    rprintln!(=>1, "DRV enabled?: {}", is_drv_on);

    let is_fault = bp.drv.fault.is_low().unwrap();
    match is_fault {
        true => {
            rprintln!(=>1, "{}DRV FAULT{}", vt100::RED, vt100::DEFAULT);
        }
        false => {
            rprintln!(=>1, "{}DRV OK{}", vt100::GREEN, vt100::DEFAULT);
        }
    }

    rprint!(=>1, "V_IN: ");
    print_phase_voltage!(bp, v_in);
    rprintln!(=>1, "\n");

    let gain = 20;

    rprintln!(=>1, "A: ");
    print_phase_voltage!(bp, v_a);
    rprintln!(=>1, "");
    print_phase_current!(bp, i_a, gain);
    rprintln!(=>1, "\n");

    rprintln!(=>1, "B: ");
    print_phase_voltage!(bp, v_b);
    rprintln!(=>1, "");
    print_phase_current!(bp, i_b, gain);
    rprintln!(=>1, "\n");

    rprintln!(=>1, "C: ");
    print_phase_voltage!(bp, v_c);
    rprintln!(=>1, "");
    print_phase_current!(bp, i_c, gain);
    rprintln!(=>1, "\n");

    let halls = bp.hall_sensors.read();
    rprintln!(=>1, "Halls: {:?}", halls);
}

pub struct Ohms(pub u32);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd)]
pub struct MilliVolts(pub i32);
impl core::fmt::Display for MilliVolts {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}mV", self.0)
    }
}
impl core::ops::Sub for MilliVolts {
    type Output = MilliVolts;

    fn sub(self, rhs: Self) -> Self::Output {
        MilliVolts(self.0 - rhs.0)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd)]
pub struct MilliAmperes(pub i32);
impl core::fmt::Display for MilliAmperes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}mA", self.0)
    }
}

pub struct MicroOhms(pub u32);

pub fn resistor_divider_inverse(rt: Ohms, rb: Ohms, vin: MilliVolts) -> MilliVolts {
    MilliVolts(vin.0 * (rt.0 as i32 + rb.0 as i32) / rb.0 as i32)
}

pub fn voltage_to_current(v_adc: MilliVolts, mid_point: MilliVolts, shunt: MicroOhms, gain: u8) -> MilliAmperes {
    let mv_at_shunt = (v_adc - mid_point).0;
    let current = (mv_at_shunt * 1_000_000) / shunt.0 as i32 / gain as i32;
    MilliAmperes(current)
}