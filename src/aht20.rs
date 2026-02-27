use arduino_hal::{I2c, prelude::{_embedded_hal_blocking_i2c_Read, _embedded_hal_blocking_i2c_Write}};

use crate::local_delay::LocalDelay;

const COMMAND_GET_STATUS: [u8; 1] = [0x71];
const COMMAND_START_MEASUREMENT: [u8; 3] = [0xAC, 0x33, 0x00];

pub struct Aht20;

pub struct Aht20MeasurementData{
    pub temperature: i16,
    pub humidity: i16,
    pub crc_passed: bool,
}

impl Aht20{
    pub fn measure_raw(i2c: &mut I2c, address: u8) -> Option<[u8; 7]>{
        if i2c.write(address, &COMMAND_START_MEASUREMENT).is_err(){
            return None;
        }

        let mut delay = LocalDelay::new();

        delay.delay_ms(100);

        let mut buffer = [0u8; 7];

        if i2c.read(address, &mut buffer).is_err(){
            return None
        }

        Some(buffer)
    }

    pub fn measure(i2c: &mut I2c, address: u8) -> Option<Aht20MeasurementData>{
        let Some(raw_data) = Self::measure_raw(i2c, address) else{
            return None;
        };

        // Get humidity
        let raw_humidity = 0i64
            | ((raw_data[1] as i64) << 12)
            | ((raw_data[2] as i64) << 4) 
            | ((raw_data[3] as i64) >> 4);
        
        let humidity = Self::humidity_from_raw(raw_humidity);

        // Get temperature
        let raw_temperature = 0i64
            | ((raw_data[3] as i64 & 0x0f) << 16) 
            | ((raw_data[4] as i64) << 8) 
            | raw_data[5] as i64;
        
        let temperature = Self::temperature_from_raw(raw_temperature);


        Some(Aht20MeasurementData { temperature, humidity, crc_passed: true })
    }

    pub fn temperature_from_raw(raw: i64) -> i16{
       (((raw * 20000) >> 20) - 5000) as i16
    }

    pub fn humidity_from_raw(raw: i64) -> i16{
       (raw * 10000 >> 20) as i16
    }
}

