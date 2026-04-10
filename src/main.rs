#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use core::u16;

use arduino_hal::{hal::usart::Usart, port::{Pin, mode::Output}, prelude::_unwrap_infallible_UnwrapInfallible, usart::Baudrate};
use panic_halt as _;

use ook_433mhz::{driver::OokDriver, mock_pin::MockPin};

mod local_clock;
use local_clock::MHz3;

use crate::{local_delay::LocalDelay, power_controlled_bus::ActiveLowPin};

mod local_delay;
mod power_controlled_bus;
mod aht20;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

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
        Baudrate::<MHz3>::new(9600)
    );

    let mut delay = LocalDelay::new();

    let mut driver = OokDriver::new(pins.d7.into_output(), MockPin::new());

    let mut counter = 0u16;

    ufmt::uwrite!(&mut serial, "Ready :3\r\n").unwrap_infallible();

    loop {
        driver.tick();
        delay.delay_ms(10);

        if driver.is_idle(){
            delay.delay_ms(3000);
            counter += 1;

            ufmt::uwrite!(&mut serial, "Sending new data: {}\r\n", counter).unwrap_infallible();

            driver.send(&counter.to_le_bytes());
        }
    }
}