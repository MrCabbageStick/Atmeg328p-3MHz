use arduino_hal::{I2c, prelude::{_embedded_hal_blocking_i2c_Write, _embedded_hal_blocking_i2c_WriteRead}};


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

        i2c.write(self.address, &[0, (COMMAND_00 & 0x0f) as u8, ((COMMAND_00 & 0xf0) >> 8) as u8])?;
        i2c.write(self.address, &[1, (COMMAND_01 & 0x0f) as u8, ((COMMAND_01 & 0xf0) >> 8) as u8])?;
        i2c.write(self.address, &[2, (COMMAND_02 & 0x0f) as u8, ((COMMAND_02 & 0xf0) >> 8) as u8])?;
        i2c.write(self.address, &[3, (COMMAND_03 & 0x0f) as u8, ((COMMAND_03 & 0xf0) >> 8) as u8])?;

        Ok(())
    }

    pub fn read(&self, i2c: &mut I2c) -> Result<u32, arduino_hal::i2c::Error>{
        let mut buffer = [0u8; 2];

        i2c.write_read(self.address, &[0x5], &mut buffer)?;

        let raw = u16::from_le_bytes([buffer[0], buffer[1]]);

        Ok(Self::convert(raw))
    }

    pub fn convert(raw: u16) -> u32{
        const MULTIPLIER: u32 = 1000;

        // (OUTPUT_DATA / ALS sensitivity) * (10 / IT [ms])
        ((raw as u32) * MULTIPLIER / 42) / (600 / 10 * MULTIPLIER) / MULTIPLIER
    }
}