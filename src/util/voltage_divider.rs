use arduino_hal::{adc::{AdcChannel, Adc}};

/// Maximal value returned from adc
const ARDUINO_ADC_MAX: u32 = 1023;

/// Calculates `Vin` for:
/// ```text
///  ┌── Vin  
/// ┌┴┐        
/// │ │R1    
/// └┬┘      
///  ├── Vout
/// ┌┴┐      
/// │ │R2    
/// └┬┘      
///  ┴       
/// ```
/// Where `Vout` is the voltage read from `pin`.
///
/// To determine the actual voltage on `pin`, this function uses
/// `VREF_MV`, which should be set to the MCU's supply voltage
/// (e.g. `3300` for 3.3 V or `5000` for 5 V).
pub fn reverse_voltage_divider_mv<
    const R1: u32, 
    const R2: u32, 
    const VREF_MV: u32,
    PIN: AdcChannel<arduino_hal::hal::Atmega, arduino_hal::pac::ADC>
>(pin: &mut PIN, adc: &mut Adc) -> u16{
    let raw = adc.read_blocking(pin) as u32;

    // Voltage at ADC pin
    let vadc = (VREF_MV * raw) / ARDUINO_ADC_MAX;

    // Reverse voltage divider
    (vadc * (R1 + R2) / R2) as u16
}