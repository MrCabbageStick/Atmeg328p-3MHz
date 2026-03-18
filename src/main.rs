#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use core::u16;

use arduino_hal::{hal::usart::Usart, port::{Pin, mode::Output}, prelude::_unwrap_infallible_UnwrapInfallible, usart::Baudrate};
use ask433::{driver::{AskDriver, AskMode}, heapless::Vec};
use panic_halt as _;

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

    let mut driver = AskDriver::new(
        pins.d7.into_output(), 
        pins.a0.into_pull_up_input(), 
        None::<Pin<Output>>, 
        4, 
        None, 
        None
    );

    let mut counter = 0u16;
    let mut data = [((counter >> 8) & 0xf) as u8, (counter & 0xf) as u8];

    ufmt::uwrite!(&mut serial, "Ready :3\r\n").unwrap_infallible();

    loop {
        driver.tick();
        delay.delay_us(100);

        if matches!(driver.mode, AskMode::Idle){
            // Why do i have to do this?
            driver.tx_buf.clear();

            ufmt::uwrite!(&mut serial, "Sending data: {}\r\n", counter).unwrap_infallible();

            let data = Vec::from_slice(&[((counter >> 8) & 0xf) as u8, (counter & 0xf) as u8]).unwrap();
            driver.send(data);

            counter += 1;
        }
        // if driver.send(Vec::from_slice(&data).unwrap()) {
        //     ufmt::uwrite!(&mut serial, "Sending data: {} -> [{}, {}]\r\n", counter, data[0], data[1]).unwrap_infallible();
        //     counter += 1;
        //     data = [((counter >> 8) & 0xf) as u8, (counter & 0xf) as u8];
        // }
        
    }
}