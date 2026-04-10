use arduino_hal::{I2c, prelude::{_embedded_hal_blocking_i2c_Write, _embedded_hal_blocking_i2c_WriteRead}};


pub struct Bmp280{
    address: u8,
    compensations: Bmp280Compensations,
}

impl Bmp280{
    // Oversampling settings [x0 - disabled, x1, x2, x4, x8, x16]
    pub const OVERSAMPLING: [u8; 6] = [0b000, 0b001, 0b010, 0b011, 0b100, 0b111];
    // Power mode setting: [Sleep, Forced, Normal]
    pub const POWER_MODES: [u8; 3] = [0b00, 0b01, 0b11];
    // Normal mode standby time settings: [0.5ms, 62.5 * 2^index]
    pub const STANDBY_TIMES: [u8; 8] = [0, 0b1, 0b10, 0b11, 0b100, 0b101, 0b110, 0b111];
    // IIR filter settings [off, 2, 4, 8, 16]
    pub const IIR_FILTER: [u8; 5] = [0, 1, 2, 3, 4];

    pub const CTRL_MEAS_REGISTER_ADDRESS: u8 = 0xf4;
    pub const CONFIG_REGISTER_ADDRESS: u8 = 0xf5;

    pub const TEMPERATURE_MS_BYTE_ADDRESS: u8 = 0xfa;
    pub const PRESSURE_MS_BYTE_ADDRESS: u8 = 0xf7;

    pub const COMPENSATION_START_ADDRESS: u8 = 0x88;
    pub const N_COMPENSATION_BYTES: usize = 24;


    pub fn init(i2c: &mut I2c, address: u8) -> Result<Self, ()>{
        // Main config
        let normal_mode_standby_time = Self::STANDBY_TIMES[0];
        let iir_filter = Self::IIR_FILTER[0];
        let isp_3wire_setting = 0u8;

        // Register layout:
        // | 3 bit standby setting | 3 bit iir filter setting | 1 bit space | 1 bit isp setting |
        let config_register_value = (normal_mode_standby_time << 5) | (iir_filter << 2) | isp_3wire_setting;
        if i2c.write(address, &[Self::CONFIG_REGISTER_ADDRESS, config_register_value]).is_err(){
            return Err(());
        }

        // Config measurements
        let temperature_oversampling = Self::OVERSAMPLING[1];
        let pressure_oversampling = Self::OVERSAMPLING[1];
        let power_mode = Self::POWER_MODES[2];

        // Register layout:
        // | 3 bit temp oversampling | 3 bit pressure oversampling | 2 bit power mode |
        let ctrl_meas_register_val = (temperature_oversampling << 5) | (pressure_oversampling << 2) | power_mode;
        if i2c.write(address, &[Self::CTRL_MEAS_REGISTER_ADDRESS, ctrl_meas_register_val]).is_err(){
            return Err(());
        }

        let Ok(compensations) = Self::read_compensation_registers(i2c, address) else {
            return Err(());
        };

        Ok(Self { address, compensations })
    }

    fn read_compensation_registers(i2c: &mut I2c, address: u8) -> Result<Bmp280Compensations, ()>{
        let mut buffer = [0u8; Self::N_COMPENSATION_BYTES];

        if i2c.write_read(address, &[Self::COMPENSATION_START_ADDRESS], &mut buffer).is_err(){
            return Err(());
        }

        Ok(Bmp280Compensations{
            dig_t1: u16::from_le_bytes([buffer[0], buffer[1]]),
            dig_t2: i16::from_le_bytes([buffer[2], buffer[3]]),
            dig_t3: i16::from_le_bytes([buffer[4], buffer[5]]),

            dig_p1: 0,
            dig_p2: 0,
            dig_p3: 0,
            dig_p4: 0,
            dig_p5: 0,
            dig_p6: 0,
            dig_p7: 0,
            dig_p8: 0,
            dig_p9: 0,
        })
    }

    pub fn read_raw(&self, i2c: &mut I2c) -> Result<Bmp280RawData, ()>{
        let mut buffer = [0u8; 6];

        if i2c.write_read(self.address, &[Self::PRESSURE_MS_BYTE_ADDRESS], &mut buffer).is_err(){
            return Err(());
        }

        // Temperature and pressure are 20 bit numbers
        return Ok(Bmp280RawData { 
            temperature: ((buffer[3] as i32) << 12) | ((buffer[4] as i32) << 4) | ((buffer[5] as i32) >> 4), 
            pressure: ((buffer[0] as u32) << 12) | ((buffer[1] as u32) << 4) | ((buffer[2] as u32) >> 4),
        })
    }

    pub fn convert_raw_temp(&self, temp: i32) -> i32{
        let var1 = ((temp >> 3) - ((self.compensations.dig_t1 as i32) << 1)) * ((self.compensations.dig_t2 as i32) >> 11);
        let var2 = (
            (
                (
                    ((temp >> 4) - (self.compensations.dig_t1 as i32)) * ((temp >> 4) - self.compensations.dig_t1 as i32)
                ) >> 12
            ) * (self.compensations.dig_t3 as i32)
        ) >> 14;

        ((var1 + var2) * 5 + 128) >> 8
    }

    pub fn read_data(&self, i2c: &mut I2c) -> Result<Bmp280Data, ()>{
        let raw = self.read_raw(i2c)?;
        
        Ok(Bmp280Data { 
            temperature: self.convert_raw_temp(raw.temperature), 
            pressure: raw.pressure,
        })
    }
}

struct Bmp280Compensations{
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

pub struct Bmp280RawData{
    pub temperature: i32,
    pub pressure: u32,
}

pub struct Bmp280Data{
    pub temperature: i32,
    pub pressure: u32,
}