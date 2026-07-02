use arduino_hal::{Adc, adc::AdcChannel, i2c};
use embedded_hal::{delay::DelayNs, digital::InputPin};
use ufmt::derive::uDebug;

use crate::{data_handling::{labeled_readout::LabeledReadout, static_labeled_readout::{Barometer, Higrometer, Luxmeter, SensorId0, SensorId1, Thermometer, TypedLabelReadout, UnitScale1_100, UnitScale1_1000}}, drivers::{aht20::{Aht20, Aht20Error}, bmp280::{self, driver::{Bmp280, Bmp280ReadError}}, veml7700::{self, driver::Veml7700}}, resistor_divider};

const BMP280_ADDR: u8 = 0x77;
const VEML7700_ADDR: u8 = 0x10;
const AHT20_ADDR: u8 = 0x38;

const SENSOR_INIT_DELAY_MS: u32 = 5;

pub struct ClimateSensor<
VemlConfig, 
Bmp280Config, 
D: DelayNs,
SumCapPin: AdcChannel<arduino_hal::hal::Atmega, arduino_hal::pac::ADC>,
SecondCapPin: AdcChannel<arduino_hal::hal::Atmega, arduino_hal::pac::ADC>,
>{
    // SENSORS
    aht20: Aht20,
    bmp280: Bmp280<Bmp280Config>,
    veml7700: Veml7700<VemlConfig>,
    // PINS
    sum_capacitor: SumCapPin,
    capacitor_2_pin: SecondCapPin,
    // COMMUNICATION
    i2c: i2c::I2c,
    // TIMING
    delay: D,
    // MISC
    sensor_id: u8,
}

impl<VemlConfig, Bmp280Config, D, SumCapPin, SecondCapPin> ClimateSensor<VemlConfig, Bmp280Config, D, SumCapPin, SecondCapPin>
where VemlConfig: veml7700::config::Config, 
    Bmp280Config: bmp280::config::Config, 
    D: DelayNs, 
    SumCapPin: AdcChannel<arduino_hal::hal::Atmega, arduino_hal::pac::ADC>,
    SecondCapPin: AdcChannel<arduino_hal::hal::Atmega, arduino_hal::pac::ADC>,
{
    pub fn new(sensor_id: u8, i2c: i2c::I2c, delay: D, sum_capacitor: SumCapPin, second_capacitor: SecondCapPin) -> Self{
        Self{
            bmp280: Bmp280::new(BMP280_ADDR),
            veml7700: Veml7700::new(VEML7700_ADDR),
            aht20: Aht20::new(AHT20_ADDR),
            i2c,
            delay,
            sum_capacitor,
            capacitor_2_pin: second_capacitor,
            sensor_id,
        }
    }

    fn sensors_init(&mut self) -> Result<(), ModuleInitError>{
        self.bmp280.init(&mut self.i2c, &mut self.delay).map_err(|err| ModuleInitError::Bmp280Error(err))?;
        self.delay.delay_ms(SENSOR_INIT_DELAY_MS);

        self.veml7700.init(&mut self.i2c).map_err(|err| ModuleInitError::Veml7700(err))?;
        self.delay.delay_ms(SENSOR_INIT_DELAY_MS);

        self.aht20.init(&mut self.i2c, &mut self.delay)?;
        self.delay.delay_ms(SENSOR_INIT_DELAY_MS);

        Ok(())
    }

    pub fn init(&mut self) -> Result<(), ClimateSensorInitError>{
        self.sensors_init()?;


        Ok(())
    }

    fn read_modules(&mut self, output: &mut [u8]) -> Result<usize, ModuleReadError>{
        const N_BYTES: usize = 25;

        if output.len() < N_BYTES{
            // TODO: Change to an error
            return Ok(0);
        }

        // TODO: Create abstraction and do everything below in a loop
        {
            let data = self.aht20.read(&mut self.i2c, &mut self.delay)?;
            let temp = TypedLabelReadout::<SensorId0, UnitScale1_100, Thermometer>::new(data.temperature as u32);
            let humid = TypedLabelReadout::<SensorId0, UnitScale1_100, Higrometer>::new(data.humidity as u32);

            // TODO: Use crc check

            output[0..5].copy_from_slice(&temp.get_bytes());
            output[5..10].copy_from_slice(&humid.get_bytes());
        }
        {
            let data = self.bmp280.read(&mut self.i2c)?;
            let temp = TypedLabelReadout::<SensorId1, UnitScale1_1000, Thermometer>::new(data.temperature as u32);
            let pres = TypedLabelReadout::<SensorId0, UnitScale1_1000, Barometer>::new(data.pressure);

            output[10..15].copy_from_slice(&temp.get_bytes());
            output[15..20].copy_from_slice(&pres.get_bytes());
        }
        {
            let data = self.veml7700.read(&mut self.i2c).map_err(|err| ModuleReadError::Veml7700(err))?;
            let luxs = TypedLabelReadout::<SensorId0, UnitScale1_100, Luxmeter>::new(data);
            output[20..25].copy_from_slice(&luxs.get_bytes());
        }

        Ok(N_BYTES)
    }

    pub fn read_bytes(&mut self) -> Result<[u8; 26], ClimateSensorReadError>{
        let mut bytes = [0; 26];
        bytes[0] = self.sensor_id;

        let _bytes_written = self.read_modules(&mut bytes[1..])?;

        Ok(bytes)
    }

    pub fn get_charge_info(&mut self, adc: &mut Adc) -> ChargeInfo{
        let sum_mv = resistor_divider::read_voltage_divider_mv::<100_000,1_000_000,3300, _>(&mut self.sum_capacitor, adc);
        let other_mv = resistor_divider::read_voltage_divider_mv::<1_000_000, 100_000,3300, _>(&mut self.capacitor_2_pin, adc);

        ChargeInfo { sum_mv, first_mv: sum_mv - other_mv, second_mv: other_mv }
    }
}

pub struct ChargeInfo{
    pub sum_mv: u16,
    pub first_mv: u16,
    pub second_mv: u16,
}

// === ERRORS ===

// Initialization
#[derive(Debug, uDebug)]
pub enum ModuleInitError{
    Aht20Error(Aht20Error),
    Veml7700(i2c::Error),
    Bmp280Error(i2c::Error),
}

impl From<Aht20Error> for ModuleInitError{ fn from(value: Aht20Error) -> Self { Self::Aht20Error(value) } }

#[derive(Debug, uDebug)]
pub enum ClimateSensorInitError{
    ModuleInitError(ModuleInitError),
}

impl From<ModuleInitError> for ClimateSensorInitError{ fn from(value: ModuleInitError) -> Self{ Self::ModuleInitError(value) } }


// Reading
#[derive(Debug, uDebug)]
pub enum ModuleReadError{
    Aht20Error(Aht20Error),
    Veml7700(i2c::Error),
    Bmp280Error(Bmp280ReadError),
}

impl From<Aht20Error> for ModuleReadError{ fn from(value: Aht20Error) -> Self { ModuleReadError::Aht20Error(value) } }
impl From<Bmp280ReadError> for ModuleReadError{ fn from(value: Bmp280ReadError) -> Self { ModuleReadError::Bmp280Error(value) } }

#[derive(Debug, uDebug)]
pub enum ClimateSensorReadError{
    ModuleReadError(ModuleReadError),
}

impl From<ModuleReadError> for ClimateSensorReadError{ fn from(value: ModuleReadError) -> Self { ClimateSensorReadError::ModuleReadError(value) } }