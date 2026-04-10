#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_hal::{I2c, hal::usart::{Usart}, prelude::{_embedded_hal_blocking_i2c_Read, _embedded_hal_blocking_i2c_Write, _unwrap_infallible_UnwrapInfallible}, usart::Baudrate};
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
        Baudrate::<MHz3>::new(9600)
    );

    let mut delay = LocalDelay::new();

    // let mut i2c = I2c::with_external_pullup(
    //     dp.TWI, 
    //     pins.a4.into_floating_input(), 
    //     pins.a5.into_floating_input(), 
    //     50_000
    // );
    let mut i2c = I2c::with_external_pullup(
        dp.TWI, 
        pins.a4.into_floating_input(), 
        pins.a5.into_floating_input(), 
        50_000
    );

    ufmt::uwrite!(&mut serial, "--------------------------\r\n").unwrap_infallible();

    let mut temp_sensors_found = false;
    
    for addr in 0u8..=127{
        match i2c.read(addr, &mut [0]) {
            Ok(_) => { 
                ufmt::uwrite!(&mut serial, "Device found on address: {}\n\r", addr).unwrap_infallible();
                if addr == 0x38{
                    temp_sensors_found = true;
                }
            },
            Err(_) => {}
        }
    }

    if temp_sensors_found{
        match i2c.write(0x38, &[0x71]){
            Ok(_) => {},
            Err(_) => { ufmt::uwrite!(&mut serial, "Unable to write to 0x38\r\n").unwrap_infallible() }
        };

        delay.delay_ms(10);

        let mut calibration_check = [0u8; 1];
        let mut callibrated = false;
        
        match i2c.read(0x38, &mut calibration_check){
            Ok(_) => { 
                ufmt::uwrite!(&mut serial, "Read status byte: {:x}\r\n", calibration_check[0]).unwrap_infallible();
                // Check 3rd bit
                ufmt::uwrite!(&mut serial, "{}\r\n", ((calibration_check[0] >> 3) & 0b1000)).unwrap_infallible();
                callibrated = ((calibration_check[0] >> 3) & 0b1) == 1
            },
            Err(_) => { ufmt::uwrite!(&mut serial, "Unable to read from 0x38\r\n").unwrap_infallible() }
        };

        if callibrated{
            ufmt::uwrite!(&mut serial, "Device 0x38 is calibrated\n\r").unwrap_infallible();
        }
        else{
            // TODO: Callibration
            ufmt::uwrite!(&mut serial, "Device 0x38 is NOT callibrated\n\r").unwrap_infallible();
        }

        // Measurement
        // match i2c.write(0x38, &[0xAC, 0x33, 0x00]){
        //     Ok(_) => { ufmt::uwrite!(&mut serial, "Send measurement command\r\n").unwrap_infallible() },
        //     Err(_) => { ufmt::uwrite!(&mut serial, "Unable to send measurement command\r\n").unwrap_infallible() }
        // };

        // delay.delay_ms(500);

        // let mut measurement_buffer = [0u8; 7];

        // match i2c.read(0x38, &mut measurement_buffer){
        //     Ok(_) => { 
        //         ufmt::uwrite!(&mut serial, "Data read:\r\n").unwrap_infallible();
        //         for byte in &measurement_buffer {
        //             ufmt::uwrite!(&mut serial, "{:x} ", *byte).unwrap_infallible();
        //         }
        //         ufmt::uwrite!(&mut serial, "\r\n").unwrap_infallible();
        //     },
        //     Err(_) => { ufmt::uwrite!(&mut serial, "Unable to read measurement data\r\n").unwrap_infallible() }
        // };

        // let raw_temp = 0i64
        //     | ((measurement_buffer[3] as i64 & 0x0f) << 16) 
        //     | ((measurement_buffer[4] as i64) << 8) 
        //     | measurement_buffer[5] as i64;

        // // let temp = (raw_temp as f32 / (1 << 20) as f32) * 200.0 - 50.0;
        // // let temp_whole_number = temp as u32;
        // // let temp_fraction = (temp - temp_whole_number as f32 * 1000.0) as u32;
        // let temp = ((raw_temp * 20000) >> 20) - 5000;
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

        ufmt::uwrite!(&mut serial, "> AHT20\r\n").unwrap_infallible();
        
        // // ufmt::uwrite!(&mut serial, "Womping...\r\n").unwrap_infallible();
        let humidity_temp_data = Aht20::measure(&mut i2c, 0x38).unwrap_or(
            Aht20MeasurementData{ temperature: 0, humidity: 0, crc_passed: false }
        );


        ufmt::uwrite!(&mut serial, "--> Temp: {}.{}^C\r\n", humidity_temp_data.temperature / 100, humidity_temp_data.temperature % 100).unwrap_infallible();
        ufmt::uwrite!(&mut serial, "--> Humidity: {}.{}%\r\n", humidity_temp_data.humidity / 100, humidity_temp_data.humidity % 100).unwrap_infallible();
        
        ufmt::uwrite!(&mut serial, "\r\n").unwrap_infallible();
        
        delay.delay_ms(5000);
    }
}