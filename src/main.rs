#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_hal::{Adc, I2c, Peripherals, hal::{delay::Delay, usart::Usart}, pac::tc1, prelude::_unwrap_infallible_UnwrapInfallible, usart::Baudrate};
use embedded_hal::delay::DelayNs;
use ook_433mhz::{driver::OokDriver, mock_pin::MockPin};
use panic_halt as _;
use arduino_hal::hal::clock::MHz8;

use battery_free_climat_sensor::{drivers::{aht20::Aht20, bmp280::{config::DefaultConfig, driver::Bmp280}, geiger_counter::GeigerCounter, veml7700::{config::ConfigFastLowPower, driver::Veml7700}}, power_controlled_bus::ActiveLowPin, resistor_divider::read_voltage_divider_mv, util::{split_fixed_point, timer::{millis, millis_init}}};

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
    let mut bmp280 = Bmp280::<DefaultConfig>::new(0x77);
    let lx_meter = Veml7700::<ConfigFastLowPower>::new(0x10);
    let aht20 = Aht20::new(0x38);

    // Inits
    delay.delay_ms(5);

    match lx_meter.init(&mut i2c){
        Ok(_) => ufmt::uwrite!(&mut serial, "VEML7700 initialized\r\n").unwrap_infallible(),
        Err(e) => ufmt::uwrite!(&mut serial, "Unable to initialize VEML7700: \n{:?}\r\n", e).unwrap_infallible()
    }

    delay.delay_ms(5);
    match aht20.init(&mut i2c, &mut delay){
        Ok(_) => ufmt::uwrite!(&mut serial, "AHT20 initialized\r\n").unwrap_infallible(),
        Err(e) => ufmt::uwrite!(&mut serial, "Unable to initialize AHT20: \n{:?}\r\n", e).unwrap_infallible()
    }

    delay.delay_ms(5);

    match bmp280.init(&mut i2c, &mut delay){
        Ok(_) => ufmt::uwrite!(&mut serial, "Bmp280 initilized\r\n").unwrap_infallible(),
        Err(e) => ufmt::uwrite!(&mut serial, "Unable to initialize Bmp280: {:?}\r\n", e).unwrap_infallible(),
    };

    geiger_counter.init();

    millis_init(dp.TC0);


    ufmt::uwrite!(&mut serial, "--------------------------\r\n").unwrap_infallible();

    let mut last_millis = millis();

    loop {
        geiger_counter.tick();

        if millis() - last_millis >= 5000{
            last_millis = millis();
            ufmt::uwrite!(&mut serial, "\r\n--------------- NEW MEASUREMENT ---------------\r\n").unwrap_infallible();

            ufmt::uwrite!(&mut serial, "Cpm: {}\r\n", geiger_counter.cpm()).unwrap_infallible();

            match lx_meter.read(&mut i2c){
                Ok(lx) => ufmt::uwrite!(&mut serial, "Light sensor: {} lx\r\n", lx).unwrap_infallible(),
                Err(_) => ufmt::uwrite!(&mut serial, "Unable to read light sensor data\r\n").unwrap_infallible()
            }

            match aht20.read(&mut i2c, &mut delay){
                Ok(data) => {
                    let temp = split_fixed_point(data.temperature, 100);
                    ufmt::uwrite!(&mut serial, "Temperature: {}.{}°C\r\n", temp.0, temp.1).unwrap_infallible();
                    let humidity = split_fixed_point(data.humidity, 100);
                    ufmt::uwrite!(&mut serial, "Relative humidity: {}.{}%\r\n", humidity.0, humidity.1).unwrap_infallible();
                },
                Err(e) => ufmt::uwrite!(&mut serial, "Unable to read Aht20 data: \n{:?}\r\n", e).unwrap_infallible()
            }

            match bmp280.read(&mut i2c){
                Ok(data) => {
                    let temp = split_fixed_point(data.temperature, 100);
                    let pressure = data.pressure / 256;

                    ufmt::uwrite!(
                        &mut serial,
                        "Temp2: {}.{}°C\r\nPressure: {}Pa\r\n", 
                        temp.0, temp.1,
                        pressure
                    ).unwrap_infallible()
                },
                Err(e) => ufmt::uwrite!(&mut serial, "Unable to read BMP280 data: {:?}\r\n", e).unwrap_infallible(),
            }

            let v_cap = read_voltage_divider_mv::<100_000,1_000_000,3300,_>(&mut capacitor_vsum_pin, &mut adc);
            let v_half_cap = read_voltage_divider_mv::<1_000_000,100_000,3300,_>(&mut capacitor_halfv_pin, &mut adc);

            ufmt::uwrite!(&mut serial, "Capacitor1: {}mV\r\n", v_cap - v_half_cap).unwrap_infallible();
            ufmt::uwrite!(&mut serial, "Capacitor2: {}mV\r\n", v_half_cap).unwrap_infallible();
        }
    }
}