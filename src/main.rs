#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_hal::{I2c, Peripherals, hal::{delay::Delay, usart::Usart}, pac::tc1, prelude::_unwrap_infallible_UnwrapInfallible, usart::Baudrate};
use embedded_hal::delay::DelayNs;
use panic_halt as _;
use arduino_hal::hal::clock::MHz8;

use battery_free_climat_sensor::{drivers::{bmp280::Bmp280, geiger_counter::{GeigerCounter}, veml7700::{config::ConfigFastLowPower, driver::Veml7700}}, power_controlled_bus::ActiveLowPin, util::timer::{millis, millis_init}};

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
        Baudrate::<MHz8>::new(9600)
    );

    let mut delay = Delay::<MHz8>::new();

    millis_init(dp.TC0);

    let mut geiger_counter = GeigerCounter::new(dp.TC1);
    geiger_counter.init();

    // let _gm_counter_pin = pins.d5.into_floating_input();

    // let tc1 = dp.TC1;

    // unsafe {
    //     tc1.tccr1a().write(|w| w.wgm1().bits(0b00));
    //     tc1.tccr1b().write(|w| {
    //         w.wgm1().bits(0b00)
    //             .cs1().bits(0b111)
    //     });
    //     tc1.tcnt1().write(|w| w.bits(0));
    // }


    ufmt::uwrite!(&mut serial, "--------------------------\r\n").unwrap_infallible();

    let mut last_millis = millis();

    loop {
        geiger_counter.tick();

        if millis() - last_millis >= 1000{
            last_millis = millis();
            ufmt::uwrite!(&mut serial, "Cpm: {}\r\nSamples: {}\r\n", geiger_counter.cpm(), geiger_counter.seconds_collected()).unwrap_infallible();
        }

        // let count = geiger_counter_read_and_reset(&tc1);

        // ufmt::uwrite!(&mut serial, "Count: {}p/s\r\n", count).unwrap_infallible();
    }
}