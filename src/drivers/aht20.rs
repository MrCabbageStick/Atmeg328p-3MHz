use core::{error::Error, fmt::{self, Debug, Display}};

use arduino_hal::{I2c, i2c, prelude::{_embedded_hal_blocking_i2c_Read, _embedded_hal_blocking_i2c_Write}};
use embedded_hal::delay::DelayNs;
use ufmt::derive::uDebug;

const DEFAULT_I2C_ADDRESS: u8 = 0x38;

const COMMAND_GET_STATUS: [u8; 1]        = [0x71];
const COMMAND_INITIALIZE: [u8; 3]        = [0xBE, 0x08, 0x00];
const COMMAND_START_MEASUREMENT: [u8; 3] = [0xAC, 0x33, 0x00];

const STATUS_CALIBRATED_BIT: u8 = 1 << 3;
const STATUS_BUSY_BIT: u8       = 1 << 7;


pub struct Aht20RawData {
    /// status + 5 data bytes + CRC
    pub bytes: [u8; 7],
}

pub struct Aht20Data {
    pub temperature: i16,
    pub humidity: i16,
    pub crc_passed: bool,
}

pub struct Aht20 {
    address: u8,
}

impl Aht20 {
    pub fn new(address: u8) -> Self {
        Self { address }
    }

    pub fn default() -> Self {
        Self::new(DEFAULT_I2C_ADDRESS)
    }

    /// Power-on init sequence (datasheet 5.4, step 1).
    ///
    /// Must be called once after power-on. Waits up to 40 ms.
    /// Checks the calibration-enable bit and sends the
    /// initialisation command if needed
    pub fn init<D: DelayNs>(&self, i2c: &mut I2c, delay: &mut D) -> Result<(), Aht20Error> {
        // Datasheet 5.4: wait at least 40 ms after power-on
        delay.delay_ms(40);

        let status = self.read_status(i2c)?;

        if status & STATUS_CALIBRATED_BIT == 0 {
            i2c.write(self.address, &COMMAND_INITIALIZE)?;
            delay.delay_ms(10);
        }

        Ok(())
    }

    /// Read the one-byte status register
    fn read_status(&self, i2c: &mut I2c) -> Result<u8, i2c::Error> {
        i2c.write(self.address, &COMMAND_GET_STATUS)?;

        let mut buf = [0u8; 1];
        i2c.read(self.address, &mut buf)?;

        Ok(buf[0])
    }

    /// Trigger measurement and return the raw 7-byte frame
    ///
    /// Checks the busy bit after the required 80 ms wait (datasheet 5.4,
    /// steps 2–3)
    pub fn read_raw<D: DelayNs>(&self, i2c: &mut I2c, delay: &mut D) -> Result<Aht20RawData, Aht20Error> {
        i2c.write(self.address, &COMMAND_START_MEASUREMENT)?;

        // Datasheet 5.4 step 3: wait 80 ms, then check busy bit
        delay.delay_ms(80);

        let status = self.read_status(i2c)?;
        if status & STATUS_BUSY_BIT != 0 {
            return Err(Aht20Error::SensorBusy);
        }

        let mut bytes = [0u8; 7];
        i2c.read(self.address, &mut bytes)?;

        Ok(Aht20RawData { bytes })
    }

    /// Trigger a measurement and return calibrated temperature and humidity
    pub fn read<D: DelayNs>(&self, i2c: &mut I2c, delay: &mut D) -> Result<Aht20Data, Aht20Error> {
        let raw = self.read_raw(i2c, delay)?;

        let crc_passed = Self::check_crc(&raw.bytes);

        // Humidity: bits [39:20] of the data payload (bytes 1-3)
        let raw_humidity = 0i64
            | ((raw.bytes[1] as i64) << 12)
            | ((raw.bytes[2] as i64) << 4)
            | ((raw.bytes[3] as i64) >> 4);

        // Temperature: bits [19:0] of the data payload (bytes 3-5)
        let raw_temperature = 0i64
            | ((raw.bytes[3] as i64 & 0x0F) << 16)
            | ((raw.bytes[4] as i64) << 8)
            | (raw.bytes[5] as i64);

        Ok(Aht20Data {
            temperature: Self::convert_temperature::<100>(raw_temperature),
            humidity: Self::convert_humidity::<100>(raw_humidity),
            crc_passed,
        })
    }

    /// T(°C) = (rwa_temp / 2^20) * 200 − 50   →   centidegrees: × 100
    pub fn convert_temperature<const PRECISION: i64>(raw: i64) -> i16 {
        (((raw * 200 * PRECISION) >> 20) - 50 * PRECISION) as i16
    }

    /// RH(%) = (raw_humidity / 2^20) * 100   →   centipercent: × 100
    pub fn convert_humidity<const PRECISION: i64>(raw: i64) -> i16 {
        ((raw * 100 * PRECISION) >> 20) as i16
    }

    /// CRC-8 check (polynomial x^8 + x^5 + x^4 + 1, initial value 0xFF).
    /// Covers bytes 0–5, byte 6 is the CRC.
    fn check_crc(bytes: &[u8; 7]) -> bool {
        let mut crc: u8 = 0xFF;
        for &byte in &bytes[..6] {
            crc ^= byte;
            for _ in 0..8 {
                if crc & 0x80 != 0 {
                    crc = (crc << 1) ^ 0x31;
                } else {
                    crc <<= 1;
                }
            }
        }
        crc == bytes[6]
    }
}

#[derive(Debug, uDebug)]
pub enum Aht20Error {
    /// Measurement triggered but sensor still reported busy
    SensorBusy,
    I2cError(i2c::Error),
}

impl Display for Aht20Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Aht20Error::SensorBusy  => write!(f, "AHT20 sensor busy"),
            Aht20Error::I2cError(e) => Debug::fmt(e, f),
        }
    }
}

impl Error for Aht20Error {}

impl From<i2c::Error> for Aht20Error {
    fn from(value: i2c::Error) -> Self {
        Self::I2cError(value)
    }
}