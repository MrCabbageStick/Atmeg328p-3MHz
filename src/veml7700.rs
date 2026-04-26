use core::marker::PhantomData;

use arduino_hal::{I2c, clock::MHz8, hal::delay::Delay, prelude::{_embedded_hal_blocking_i2c_Write, _embedded_hal_blocking_i2c_WriteRead}};
use embedded_hal::delay::DelayNs;

pub struct Veml7700<SM, IT>{
    address: u8,
    _sm: PhantomData<SM>,
    _it: PhantomData<IT>,
}

impl<SM: config::AlsSm, IT: config::AlsIt> Veml7700<SM, IT>{
    const CONFIG: u16 = 
        (0b000 << 13) // Reserved bits
        | SM::BITS
        | IT::BITS
        | (0b00 << 4) // Persistence
        | (0 << 1) // Disable interrupt
        | 0; // ALS on

    pub fn new(address: u8) -> Self{ Self{ address, _sm: PhantomData, _it: PhantomData } }

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

pub mod config{
    // Sensitivity mode settings
    pub trait AlsSm{
        const BITS: u16; // Bits in config register
        // Scaling for calculations to avoid floating point arithmetics
        const SCALE_NUM: u32;
        const SCALE_DEN: u32;
    }

    pub struct Sm1; // Sensitivity: 1
    pub struct Sm2; // Sensitivity: 2
    pub struct Sm1_4; // Sensitivity 1/4
    pub struct Sm1_8; // Sensitivity: 1/8

    impl AlsSm for Sm1{   const BITS: u16 = 0b00 << 11; const SCALE_DEN: u32 = 1000; const SCALE_NUM: u32 = 1000; }
    impl AlsSm for Sm2{   const BITS: u16 = 0b01 << 11; const SCALE_DEN: u32 = 1000; const SCALE_NUM: u32 = 42;   }
    impl AlsSm for Sm1_4{ const BITS: u16 = 0b10 << 11; const SCALE_DEN: u32 = 1000; const SCALE_NUM: u32 = 336;  }
    impl AlsSm for Sm1_8{ const BITS: u16 = 0b11 << 11; const SCALE_DEN: u32 = 1000; const SCALE_NUM: u32 = 168;  }

    // Integration time settings
    pub trait AlsIt {
        const BITS: u16; // Bits in config register
        const IT_MS: u32; // Time in ms
    }

    pub struct It100;
    pub struct It200;
    pub struct It400;
    pub struct It800;

    impl AlsIt for It100 { const BITS: u16 = 0b0000 << 6; const IT_MS: u32 = 100; }
    impl AlsIt for It200 { const BITS: u16 = 0b0001 << 6; const IT_MS: u32 = 200; }
    impl AlsIt for It400 { const BITS: u16 = 0b0010 << 6; const IT_MS: u32 = 400; }
    impl AlsIt for It800 { const BITS: u16 = 0b0011 << 6; const IT_MS: u32 = 800; }
}