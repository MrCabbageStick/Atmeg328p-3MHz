#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_hal::{I2c, hal::{clock::MHz8, usart::Usart}, prelude::{_embedded_hal_blocking_i2c_Read, _embedded_hal_blocking_i2c_Write, _embedded_hal_blocking_i2c_WriteRead, _unwrap_infallible_UnwrapInfallible}, usart::Baudrate};
use panic_halt as _;

mod local_clock;
use local_clock::MHz3;

use crate::{aht20::{Aht20, Aht20MeasurementData}, bmp280::Bmp280, local_delay::LocalDelay, power_controlled_bus::ActiveLowPin};

mod local_delay;
mod power_controlled_bus;
mod aht20;
mod bmp280;

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

    let mut delay = LocalDelay::new();

    let mut i2c = I2c::with_external_pullup(
        dp.TWI, 
        pins.a4.into_floating_input(), 
        pins.a5.into_floating_input(), 
        50_000
    );

    let mut buff = [0u8; 1];
    match i2c.write_read(0x77, &[0xD0], &mut buff){
        Ok(_) => ufmt::uwrite!(&mut serial, "{:x}\r\n", buff[0]).unwrap_infallible(),
        Err(e) => ufmt::uwrite!(&mut serial, "{:?}\r\n", e).unwrap_infallible(),
    }

    let bmp280 = match Bmp280::init(&mut i2c, 0x77){
        Ok(device) => {
            ufmt::uwrite!(&mut serial, "Bmp280 initilized\r\n").unwrap_infallible();
            Some(device)
        },
        Err(_) => {
            ufmt::uwrite!(&mut serial, "Unable to initialize Bmp280\r\n").unwrap_infallible();
            None
        }
    };

    loop {
        ufmt::uwrite!(&mut serial, "> BMP280\r\n").unwrap_infallible();

        if let Some(device) = &bmp280{
            match device.read_data(&mut i2c){
                Err(_) => ufmt::uwrite!(&mut serial, "--> Unable to read BMP280 data\r\n").unwrap_infallible(),
                Ok(data) => {
                    ufmt::uwrite!(&mut serial, "--> Temperature: {}m°C\r\n", data.temperature).unwrap_infallible();
                    ufmt::uwrite!(&mut serial, "--> Pressure: {}Pa\r\n", data.pressure >> 8).unwrap_infallible();
                }
            }
        }
        
        ufmt::uwrite!(&mut serial, "\r\n").unwrap_infallible();
        
        delay.delay_ms(5000);
    }
}