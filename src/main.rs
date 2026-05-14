#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_hal::{Adc, I2c, Peripherals, hal::{delay::Delay, usart::Usart}, pac::tc1, prelude::_unwrap_infallible_UnwrapInfallible, usart::Baudrate};
use embedded_hal::delay::DelayNs;
use ook_433mhz::{driver::OokDriver, mock_pin::MockPin};
use panic_halt as _;
use arduino_hal::hal::clock::MHz8;

use battery_free_climat_sensor::{data_handling::{labeled_readout::LabeledReadout, static_labeled_readout::{Luxmeter, SensorId0, Thermometer, TypedLabelReadout, UnitScale1, UnitScale1_1000}}, drivers::{aht20::Aht20, bmp280::{config::DefaultConfig, driver::Bmp280}, geiger_counter::GeigerCounter, veml7700::{config::ConfigFastLowPower, driver::Veml7700}}, power_controlled_bus::ActiveLowPin, resistor_divider::read_voltage_divider_mv, util::{split_fixed_point, timer::{millis, millis_init}}};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut en_vcc_rf = ActiveLowPin::from_pin(pins.d6.into_output());
    let mut en_vcc_i2c = ActiveLowPin::from_pin(pins.d8.into_output());

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


    let lx_meter = Veml7700::<ConfigFastLowPower>::new(0x10);

    ufmt::uwrite!(&mut serial, "--------------------------\r\n").unwrap_infallible();

    match lx_meter.init(&mut i2c){
        Ok(_) => ufmt::uwrite!(&mut serial, "VEML7700 initialized\r\n").unwrap_infallible(),
        Err(e) => ufmt::uwrite!(&mut serial, "Unable to initialize VEML7700: \r\n{:?}\r\n", e).unwrap_infallible()
    }


    let mut radio = OokDriver::new(pins.d7.into_output(), MockPin::new());

    millis_init(dp.TC0);

    let mut last_millis = millis();
    let mut wait_time = 0;

    loop {
        if millis() - last_millis >= 5 + wait_time{
            last_millis = millis();
            wait_time = 0;
            
            if radio.is_idle(){

                match lx_meter.read(&mut i2c){
                    Ok(lx) =>{
                        let data = TypedLabelReadout::<SensorId0, UnitScale1, Luxmeter>::new(lx);
                        let bytes = data.get_bytes();
                        ufmt::uwrite!(&mut serial, "Sending: {:?}\r\n", bytes).unwrap_infallible();

                        radio.send(&bytes);
                    },
                    Err(_) => ufmt::uwrite!(&mut serial, "Unable to read light sensor data\r\n").unwrap_infallible()
                }

                wait_time = 1000;
            }

            radio.tick();
        }
    }
}