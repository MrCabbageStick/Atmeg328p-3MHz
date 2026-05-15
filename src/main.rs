#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_hal::{Adc, I2c, Peripherals, hal::{delay::Delay, usart::Usart}, pac::tc1, prelude::_unwrap_infallible_UnwrapInfallible, usart::Baudrate};
use embedded_hal::delay::DelayNs;
use panic_halt as _;
use arduino_hal::hal::clock::MHz8;

use battery_free_climat_sensor::{drivers::{aht20::Aht20, bmp280::{config::DefaultConfig, driver::Bmp280}, geiger_counter::GeigerCounter, veml7700::{config::ConfigFastLowPower, driver::Veml7700}}, power_controlled_bus::ActiveLowPin, resistor_divider::read_voltage_divider_mv, util::{split_fixed_point, timer::{millis, millis_init}}};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut en_vcc_i2c = ActiveLowPin::from_pin(pins.d8.into_output());
    en_vcc_i2c.set_active();

    // Communication
    let mut serial = Usart::new(
        dp.USART0, 
        pins.d0, 
        pins.d1.into_output(),
        Baudrate::<MHz8>::new(9600)
    );

    let mut i2c = I2c::with_external_pullup(
        dp.TWI, 
        pins.a4.into_floating_input(), 
        pins.a5.into_floating_input(), 
        50_000
    );

    let mut delay = Delay::<MHz8>::new();

    // Sensors
    let mut bmp280 = Bmp280::<DefaultConfig>::new(0x77);

    delay.delay_ms(100);

    // Inits
    match bmp280.init(&mut i2c, &mut delay){
        Ok(_) => ufmt::uwrite!(&mut serial, "Bmp280 initilized\r\n").unwrap_infallible(),
        Err(e) => ufmt::uwrite!(&mut serial, "Unable to initialize Bmp280: {:?}\r\n", e).unwrap_infallible(),
    };

    millis_init(dp.TC0);


    ufmt::uwrite!(&mut serial, "--------------------------\r\n").unwrap_infallible();

    let mut last_millis = millis();

    loop {

        if millis() - last_millis >= 1000{
            last_millis = millis();
            ufmt::uwrite!(&mut serial, "\r\n--------------- NEW MEASUREMENT ---------------\r\n").unwrap_infallible();

            match bmp280.read(&mut i2c){
                Ok(data) => {
                    let pressure_whole = data.pressure / 256;
                    let pressure_altitude_compensated = (pressure_whole * 1024) / 1000;

                    let temp = split_fixed_point(data.temperature, 100);
                    ufmt::uwrite!(
                        &mut serial,
                        "Temp2: {}.{}°C\r\nPressure: {}Pa\r\n", 
                        temp.0, temp.1,
                        pressure_altitude_compensated
                    ).unwrap_infallible()
                },
                Err(e) => ufmt::uwrite!(&mut serial, "Unable to read BMP280 data: {:?}\r\n", e).unwrap_infallible(),
            }
        }
    }
}