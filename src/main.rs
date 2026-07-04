#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_hal::{Adc, hal::usart::Usart, prelude::_unwrap_infallible_UnwrapInfallible, usart::Baudrate};
use panic_halt as _;
use arduino_hal::hal::clock::MHz8;

use battery_free_climat_sensor::sleep;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut adc = Adc::new(dp.ADC, Default::default());

    let mut led = pins.d10.into_output();

    pins.d9.into_output_high();
    pins.d8.into_output_high();
    pins.d6.into_output_high();

    sleep::setup_wdt(&dp.CPU, &dp.WDT);
    unsafe { avr_device::interrupt::enable() };

    // Communication
    let mut serial = Usart::new(
        dp.USART0, 
        pins.d0, 
        pins.d1.into_output(),
        Baudrate::<MHz8>::new(9600)
    );

    ufmt::uwrite!(&mut serial, "--~~==### POWER ON ##==~~--\r\n").unwrap_infallible();

    loop {
        // led.toggle();
        sleep::enter_sleep(&dp.CPU);

        if !sleep::ready(){
            continue;
        }

        led.toggle();
    }
}