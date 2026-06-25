#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_hal::{Adc, I2c, hal::{delay::Delay, port::{PC0, PC1}, usart::Usart}, port::{Pin, mode::Analog}, prelude::_unwrap_infallible_UnwrapInfallible, usart::Baudrate};
use embedded_hal::delay::DelayNs;
use panic_halt as _;
use arduino_hal::hal::clock::MHz8;

use battery_free_climat_sensor::{climate_sensor::ClimateSensor, data_handling::{dynamic_labeled_readout::DynamicLabeledReadout, labeled_readout::LabeledReadout}, drivers::geiger_counter::GeigerCounter, power_controlled_bus::ActiveLowPin, util::timer::millis_init};
use battery_free_climat_sensor::drivers::{bmp280::config::DefaultConfig as Bmp280DefaultConf, veml7700::config::ConfigFastLowPower as Veml7700DefaultConf};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut adc = Adc::new(dp.ADC, Default::default());

    let mut delay = Delay::<MHz8>::new();

    // Pins for reading capacitor voltage level
    let mut capacitor_vsum_pin = pins.a1.into_analog_input(&mut adc);
    let mut capacitor_halfv_pin = pins.a0.into_analog_input(&mut adc);

    // Enable power to all devices
    let mut en_vcc_i2c = ActiveLowPin::from_pin(pins.d8.into_output());
    let mut en_vcc_rf = ActiveLowPin::from_pin(pins.d6.into_output());
    let mut en_vcc_1_wire = ActiveLowPin::from_pin(pins.d9.into_output());
    let mut en_vcc_geiger = ActiveLowPin::from_pin(pins.d10.into_output());

    en_vcc_1_wire.set_active();
    en_vcc_geiger.set_active();
    en_vcc_i2c.set_active();
    en_vcc_rf.set_active();

    // Communication
    let mut serial = Usart::new(
        dp.USART0, 
        pins.d0, 
        pins.d1.into_output(),
        Baudrate::<MHz8>::new(9600)
    );

    let mut i2c = I2c::with_external_pullup(
        dp.TWI, 
        pins.a4.into_floating_input(), 
        pins.a5.into_floating_input(), 
        50_000
    );

    // Sensors
    let mut geiger_counter = GeigerCounter::new(dp.TC1);
    millis_init(dp.TC0);


    let mut climate_sensor = ClimateSensor::<
        Veml7700DefaultConf,
        Bmp280DefaultConf, _, _, _
    >::new(
        i2c,
        Delay::<MHz8>::new(),
        capacitor_vsum_pin,
        capacitor_halfv_pin,
    );

    ufmt::uwrite!(&mut serial, "--~~==### POWER ON ##==~~--\r\n").unwrap_infallible();

    let initialized;
    match climate_sensor.init(){
        Ok(_) => {
            initialized = true;
            ufmt::uwrite!(&mut serial, "Climate sensor initilized\r\n").unwrap_infallible();
        }
        Err(err) => {
            initialized = false;
            ufmt::uwrite!(&mut serial, "Climate sensor initilized. Err: {:?}\r\n", err).unwrap_infallible();
        }
    }

    loop {
        if !initialized{ continue; }

        ufmt::uwrite!(&mut serial, " -<< NEW READOUT >>-\r\n").unwrap_infallible();

        match climate_sensor.read_bytes(){
            Ok(data) => {
                for readout_bytes in data.chunks(5){
                    match DynamicLabeledReadout::from_bytes(readout_bytes){
                        Some(readout) => ufmt::uwrite!(
                            &mut serial, 
                            "SensorId: {}, UnitScale: {}, Type: {:?}, Data: {}\r\n",
                            readout.sensor_id(),
                            readout.unit_scale(),
                            readout.sensor_type(),
                            readout.get_data() as i32,
                        ).unwrap_infallible(),
                        None => ufmt::uwrite!(&mut serial, "Unable to get labeled readout from bytes\r\n").unwrap_infallible()
                    }
                }
            },
            Err(err) => {
                ufmt::uwrite!(&mut serial, "Error while reading the data: {:?}\r\n", err).unwrap_infallible()
            }
        }

        delay.delay_ms(1000);
    }
}