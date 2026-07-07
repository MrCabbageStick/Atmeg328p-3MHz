use core::{error::Error, fmt::{Debug, Display}, marker::PhantomData};

use arduino_hal::{I2c, i2c, prelude::{_embedded_hal_blocking_i2c_Write, _embedded_hal_blocking_i2c_WriteRead}};
use embedded_hal::delay::DelayNs;
use ufmt::derive::uDebug;

use crate::drivers::bmp280::config::Config;

const CTRL_MEAS_REGISTER_ADDRESS: u8 = 0xf4;
const CONFIG_REGISTER_ADDRESS: u8 = 0xf5;

// const TEMPERATURE_MS_BYTE_ADDRESS: u8 = 0xfa;
const PRESSURE_MS_BYTE_ADDRESS: u8 = 0xf7;

const COMPENSATION_START_ADDRESS: u8 = 0x88;
const N_COMPENSATION_BYTES: usize = 24;

pub struct Bmp280<CONFIG>{
    address: u8,
    compensations: Option<Bmp280Compensations>,
    _config: PhantomData<CONFIG>,
}

impl<CONFIG: Config> Bmp280<CONFIG>{
    const CTRL_MEAS_VALUE: u8 = CONFIG::CTRL_MEAS_VALUE;
    const CONFIG_VALUE: u8 = CONFIG::CONFIG_VALUE;

    pub fn new(address: u8) -> Self {
        Self { address, compensations: None, _config: PhantomData }
    }

    pub fn init<D: DelayNs>(&mut self, i2c: &mut I2c, delay: &mut D) -> Result<(), i2c::Error>{
        i2c.write(self.address, &[CTRL_MEAS_REGISTER_ADDRESS, Self::CTRL_MEAS_VALUE])?;
        i2c.write(self.address, &[CONFIG_REGISTER_ADDRESS, Self::CONFIG_VALUE])?;

        delay.delay_ms(2);

        self.compensations = Some(Bmp280Compensations::read_from_i2c(self.address, i2c)?);

        Ok(())
    }

    pub fn read_raw(&self, i2c: &mut I2c) -> Result<Bmp280RawData, i2c::Error>{
        let mut buffer = [0u8; 6];

        i2c.write_read(self.address, &[PRESSURE_MS_BYTE_ADDRESS], &mut buffer)?;

        // Raw temperature are both 20 bit values
        Ok(Bmp280RawData { 
            temperature: ((buffer[3] as i32) << 12) | ((buffer[4] as i32) << 4) | ((buffer[5] as i32) >> 4), 
            pressure: ((buffer[0] as i32) << 12) | ((buffer[1] as i32) << 4) | ((buffer[2] as i32) >> 4),
        })
    }

    /// Returns (temperature, fine_temp)
    fn convert_raw_temp(&self, temp: i32) -> Result<(i32, i32), Bmp280ReadError>{
        let compensations = self.compensations.as_ref().ok_or(Bmp280ReadError::NotInitialized)?;

        let var1 = (((temp >> 3) - ((compensations.dig_t1 as i32) << 1)) 
           * (compensations.dig_t2 as i32)) >> 11;
        let var2 = (
            (
                (
                    ((temp >> 4) - (compensations.dig_t1 as i32)) * ((temp >> 4) - compensations.dig_t1 as i32)
                ) >> 12
            ) * (compensations.dig_t3 as i32)
        ) >> 14;

        let t_fine = var1 + var2;

        Ok(((t_fine * 5 + 128) >> 8, t_fine))
    }

     pub fn convert_raw_pressure(&self, pressure: i32, fine_temp: i32) -> Result<u32, Bmp280ReadError>{
        let c = self.compensations.as_ref().ok_or(Bmp280ReadError::NotInitialized)?;

        let mut var1: i64 = (fine_temp as i64) - 128000;
        let mut var2: i64 = var1 * var1 * (c.dig_p6 as i64);
        var2 = var2 + ((var1 * (c.dig_p5 as i64)) << 17);
        var2 = var2 + ((c.dig_p4 as i64) << 35);
        var1 = ((var1 * var1 * (c.dig_p3 as i64)) >> 8) + ((var1 * (c.dig_p2 as i64)) << 12);
        var1 = ((((1i64 << 47) + var1)) * (c.dig_p1 as i64)) >> 33;

        if var1 == 0 {
            return Ok(0); // avoid division by zero
        }

        let mut p: i64 = 1048576 - (pressure as i64);
        p = (((p << 31) - var2) * 3125) / var1;
        var1 = ((c.dig_p9 as i64) * (p >> 13) * (p >> 13)) >> 25;
        var2 = ((c.dig_p8 as i64) * p) >> 19;
        p = ((p + var1 + var2) >> 8) + ((c.dig_p7 as i64) << 4);

        Ok(p as u32)
    }

    pub fn convert_from_q24_8(value: u32) -> u32{
        const ACCURACY: u32 = 1000;
        let fraction = ((value & 0xff) * ACCURACY) >> 8;

        (value >> 8) * ACCURACY + fraction
    }

    pub fn read(&self, i2c: &mut I2c) -> Result<Bmp280Data, Bmp280ReadError>{
        let raw = self.read_raw(i2c)?;
        let (temperature, fine_temp) = self.convert_raw_temp(raw.temperature)?;
        let pressure = Self::convert_from_q24_8(
            self.convert_raw_pressure(raw.pressure, fine_temp)?
        );

        Ok(Bmp280Data { temperature, pressure })
    } 
}

#[derive(Debug, uDebug)]
pub enum Bmp280ReadError{
    NotInitialized,
    I2cError(i2c::Error),
}

impl Display for Bmp280ReadError{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self{
            Bmp280ReadError::NotInitialized => write!(f, "Device not initialized"),
            Bmp280ReadError::I2cError(e) => Debug::fmt(e, f),
        }
    }
}

impl Error for Bmp280ReadError{}

impl From<i2c::Error> for Bmp280ReadError{
    fn from(value: i2c::Error) -> Self {
        Self::I2cError(value)
    }
}

pub struct Bmp280RawData{
    pub temperature: i32,
    pub pressure: i32,
}

pub struct Bmp280Data{
    pub temperature: i32,
    pub pressure: u32,
}

pub struct Bmp280Compensations{
    // Temperature compensations
    dig_t1: u16,
    dig_t2: i16,
    dig_t3: i16,
    // Pressure compensations
    dig_p1: u16,
    dig_p2: i16,
    dig_p3: i16,
    dig_p4: i16,
    dig_p5: i16,
    dig_p6: i16,
    dig_p7: i16,
    dig_p8: i16,
    dig_p9: i16,
}

impl Bmp280Compensations{
    pub fn read_from_i2c(address: u8, i2c: &mut I2c) -> Result<Self, i2c::Error>{
        let mut buffer = [0u8; N_COMPENSATION_BYTES];

        i2c.write_read(address, &[COMPENSATION_START_ADDRESS], &mut buffer)?;

        Ok(Self{
            dig_t1: u16::from_le_bytes([buffer[0], buffer[1]]),
            dig_t2: i16::from_le_bytes([buffer[2], buffer[3]]),
            dig_t3: i16::from_le_bytes([buffer[4], buffer[5]]),

            dig_p1: u16::from_le_bytes([buffer[6], buffer[7]]),
            dig_p2: i16::from_le_bytes([buffer[8], buffer[9]]),
            dig_p3: i16::from_le_bytes([buffer[10], buffer[11]]),
            dig_p4: i16::from_le_bytes([buffer[12], buffer[13]]),
            dig_p5: i16::from_le_bytes([buffer[14], buffer[15]]),
            dig_p6: i16::from_le_bytes([buffer[16], buffer[17]]),
            dig_p7: i16::from_le_bytes([buffer[18], buffer[19]]),
            dig_p8: i16::from_le_bytes([buffer[20], buffer[21]]),
            dig_p9: i16::from_le_bytes([buffer[22], buffer[23]]),
        })
    }
}