#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_hal::{Adc, I2c, hal::{delay::Delay, port::{PC0, PC1}, usart::Usart}, port::{Pin, mode::Analog}, usart::Baudrate};
use embedded_hal::digital::InputPin;
use panic_halt as _;
use arduino_hal::hal::clock::MHz8;

use battery_free_climat_sensor::{climate_sensor::ClimateSensor, drivers::geiger_counter::GeigerCounter, power_controlled_bus::ActiveLowPin, util::timer::millis_init};
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
        Bmp280DefaultConf,
        _, Pin<Analog, PC1>, Pin<Analog, PC0>
    >::new(
        i2c, 
        delay, 
        capacitor_vsum_pin.into(),
        capacitor_halfv_pin.into(),
    );

    let initialized;
    match climate_sensor.init(){
        Ok(_) => {
            initialized = true;
            ufmt::uwrite!(&mut serial, "Climate sensor initilized\r\n");
        }
        Err(err) => {
            initialized = false;
            ufmt::uwrite!(&mut serial, "Climate sensor initilized.\r\nErr: {:?}", err);
        }
    }

    loop {
        
    }
}