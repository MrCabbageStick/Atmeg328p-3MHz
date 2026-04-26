use core::marker::PhantomData;

use arduino_hal::{I2c, clock::MHz8, hal::delay::Delay, prelude::{_embedded_hal_blocking_i2c_Write, _embedded_hal_blocking_i2c_WriteRead}};
use embedded_hal::delay::DelayNs;

use crate::veml7700::config::Config;

const VALUE_REGISTER_INDEX: u8 = 0x05;

pub struct Veml7700<CONFIG>{
    address: u8,
    _config: PhantomData<CONFIG>,
}

impl<CONFIG: Config> Veml7700<CONFIG>{
    const REGISTERS: [u16; 4] = [
        CONFIG::BITS_0x00,
        CONFIG::BITS_0x01,
        CONFIG::BITS_0x02,
        CONFIG::BITS_0x03,
    ];

    const LUX_NUM: u32 = CONFIG::LUX_NUM;
    const LUX_DEN: u32 = CONFIG::LUX_DEN;

    pub fn new(address: u8) -> Self{ Self{ address, _config: PhantomData } }

    pub fn init(&self, i2c: &mut I2c) -> Result<(), arduino_hal::i2c::Error>{
        for i in 0..Self::REGISTERS.len()
        {
            let value = Self::REGISTERS[i];
            let lsb = (value & 0xFF) as u8;
            let msb = (value >> 8) as u8;

            i2c.write(self.address, &[i as u8, lsb, msb])?;
            // Add delay for write
            Delay::<MHz8>::new().delay_ms(10);
        }

        Ok(())
    }

    pub fn read(&self, i2c: &mut I2c) -> Result<u32, arduino_hal::i2c::Error>{
        let mut buffer = [0u8; 2];

        i2c.write_read(self.address, &[VALUE_REGISTER_INDEX], &mut buffer)?;

        let raw = u16::from_le_bytes([buffer[0], buffer[1]]);

        Ok(Self::raw_to_lux(raw))
    }

    fn raw_to_lux(raw: u16) -> u32{
        // Scaled conversion function from datasheet
        // Scaled to avoid floating point arithmetics
        (raw as u32 * Self::LUX_NUM) / Self::LUX_DEN
    }
}