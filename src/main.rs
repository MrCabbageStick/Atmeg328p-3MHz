#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_hal::{Adc, I2c, hal::{delay::Delay, usart::Usart}, prelude::_unwrap_infallible_UnwrapInfallible, usart::Baudrate};
use embedded_hal::delay::DelayNs;
use panic_halt as _;
use arduino_hal::hal::clock::MHz8;

use battery_free_climat_sensor::{drivers::{bmp280::Bmp280, veml7700::{config::ConfigFastLowPower, driver::Veml7700}}, power_controlled_bus::ActiveLowPin, resistor_divider::read_voltage_divider_mv};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut adc = Adc::new(dp.ADC, Default::default());

    // Pins for reading capacitor voltage level
    let mut capacitor_vsum_pin = pins.a1.into_analog_input(&mut adc);
    let mut capacitor_halfv_pin = pins.a0.into_analog_input(&mut adc);

    // Enable power to all devices
    let mut en_vcc_i2c = ActiveLowPin::from_pin(pins.d8.into_output());
    let mut en_vcc_rf = ActiveLowPin::from_pin(pins.d6.into_output());
    let mut en_vcc_1_wire = ActiveLowPin::from_pin(pins.d9.into_output());
    let mut en_vcc_geiger = ActiveLowPin::from_pin(pins.d10.into_output());

    en_vcc_1_wire.set_active();
    en_vcc_geiger.set_active();
    en_vcc_i2c.set_active();
    en_vcc_rf.set_active();


    let mut serial = Usart::new(
        dp.USART0, 
        pins.d0, 
        pins.d1.into_output(),
        Baudrate::<MHz8>::new(9600)
    );

    let mut delay = Delay::<MHz8>::new();

    ufmt::uwrite!(&mut serial, "--------------------------\r\n").unwrap_infallible();

    loop {
        let v_cap = read_voltage_divider_mv::<100_000,1_000_000,3300,_>(&mut capacitor_vsum_pin, &mut adc);
        let v_half_cap = read_voltage_divider_mv::<1_000_000,100_000,3300,_>(&mut capacitor_halfv_pin, &mut adc);
        
        ufmt::uwrite!(&mut serial, "V_cap: {}mV\r\n", v_cap).unwrap_infallible();
        ufmt::uwrite!(&mut serial, "V_half_cap: {}mV\r\n", v_half_cap).unwrap_infallible();
        ufmt::uwrite!(&mut serial, "---\r\n").unwrap_infallible();

        delay.delay_ms(1000);
    }
}