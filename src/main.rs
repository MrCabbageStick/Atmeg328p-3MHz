#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_hal::{hal::usart::Usart, prelude::_unwrap_infallible_UnwrapInfallible, usart::Baudrate};
use panic_halt as _;

mod local_clock;
use local_clock::MHz3;

use crate::local_delay::LocalDelay;

mod local_delay;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);



    /*
     * For examples (and inspiration), head to
     *
     *     https://github.com/Rahix/avr-hal/tree/main/examples
     *
     * NOTE: Not all examples were ported to all boards!  There is a good chance though, that code
     * for a different board can be adapted for yours.  The Arduino Uno currently has the most
     * examples available.
     */

    let mut serial = Usart::new(
        dp.USART0, 
        pins.d0, 
        pins.d1.into_output(),
        Baudrate::<MHz3>::new(9600)
    );


    let mut led = pins.d13.into_output();

    let mut delay = LocalDelay::new();

    loop {
        led.toggle();
        ufmt::uwrite!(&mut serial, "Womping...\r\n").unwrap_infallible();
        delay.delay_ms(1000);
    }
}
