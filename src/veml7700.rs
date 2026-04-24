use arduino_hal::{I2c, clock::MHz8, hal::delay::Delay, prelude::{_embedded_hal_blocking_i2c_Write, _embedded_hal_blocking_i2c_WriteRead}};
use embedded_hal::delay::DelayNs;

use crate::local_delay::LocalDelay;


pub struct Veml7700{
    address: u8,
}

impl Veml7700{
    pub fn new(address: u8) -> Self{ Self{ address } }

    pub fn init(&self, i2c: &mut I2c) -> Result<(), arduino_hal::i2c::Error>{
        const COMMAND_00: u16 = 0b000_01_0_0000_00_00_0_0;
        const COMMAND_01: u16 = 0b0;
        const COMMAND_02: u16 = 0b0;
        const COMMAND_03: u16 = 0b0000000000000_00_1;

        for (reg, cmd) in [
            (0x00, COMMAND_00),
            (0x01, COMMAND_01),
            (0x02, COMMAND_02),
            (0x03, COMMAND_03),
        ] {
            let lsb = (cmd & 0xFF) as u8;
            let msb = (cmd >> 8) as u8;

            i2c.write(self.address, &[reg, lsb, msb])?;
            Delay::<MHz8>::new().delay_ms(10);
        }

        Ok(())
    }

    pub fn read(&self, i2c: &mut I2c) -> Result<u32, arduino_hal::i2c::Error>{
        let mut buffer = [0u8; 2];

        i2c.write_read(self.address, &[0x5], &mut buffer)?;

        let raw = u16::from_le_bytes([buffer[0], buffer[1]]);

        Ok(Self::raw_to_lux(raw))
    }

    fn raw_to_lux(raw: u16) -> u32{
        // Scaled conversion function from datasheet
        // Scaled to avoid floating point arithmatics
        (raw as u32 * 42) / 1000
    }
}