#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_hal::{Adc, I2c, Peripherals, hal::{delay::Delay, usart::Usart}, pac::tc1, prelude::_unwrap_infallible_UnwrapInfallible, usart::Baudrate};
use embedded_hal::delay::DelayNs;
use ook_433mhz::{driver::OokDriver, mock_pin::MockPin};
use ook_433mhz::{driver::OokDriver, mock_pin::MockPin};
use panic_halt as _;
use arduino_hal::hal::clock::MHz8;

use battery_free_climat_sensor::{drivers::{aht20::Aht20, bmp280::{config::DefaultConfig, driver::Bmp280}, geiger_counter::GeigerCounter, veml7700::{config::ConfigFastLowPower, driver::Veml7700}}, power_controlled_bus::ActiveLowPin, resistor_divider::read_voltage_divider_mv, util::{split_fixed_point, timer::{millis, millis_init}}};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut en_vcc_rf = ActiveLowPin::from_pin(pins.d6.into_output());
    en_vcc_rf.set_active();

    // Communication
    let mut serial = Usart::new(
        dp.USART0, 
        pins.d0, 
        pins.d1.into_output(),
        Baudrate::<MHz8>::new(9600)
    );


    let mut radio = OokDriver::new(pins.d7.into_output(), MockPin::new());
    let mut data = 0u16;

    millis_init(dp.TC0);

    ufmt::uwrite!(&mut serial, "--------------------------\r\n").unwrap_infallible();

    let mut last_millis = millis();

    loop {
        if millis() - last_millis >= 10{
            last_millis = millis();
            
            if radio.is_idle(){
                ufmt::uwrite!(&mut serial, "Sending: {}\r\n", data).unwrap_infallible();
                radio.send(&data.to_le_bytes());
                data += 1;
            }

            radio.tick();
        }
    }
}