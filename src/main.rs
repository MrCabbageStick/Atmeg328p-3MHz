#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_hal::{I2c, hal::{delay::Delay, usart::Usart}, prelude::{_unwrap_infallible_UnwrapInfallible}, usart::Baudrate};
use embedded_hal::delay::DelayNs;
use ook_433mhz::{driver::OokDriver, mock_pin::MockPin};
use panic_halt as _;
use arduino_hal::hal::clock::MHz8;

use battery_free_climat_sensor::{data_handling::{labeled_readout::LabeledReadout, static_labeled_readout::{SensorId0, Thermometer, TypedLabelReadout, UnitScale1}}, drivers::{bmp280::Bmp280, veml7700::{config::ConfigFastLowPower, driver::Veml7700}}, power_controlled_bus::ActiveLowPin};

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // Enable power to all devices
    let mut en_vcc_i2c = ActiveLowPin::from_pin(pins.d8.into_output());
    let mut en_vcc_rf = ActiveLowPin::from_pin(pins.d6.into_output());
    let mut en_vcc_1_wire = ActiveLowPin::from_pin(pins.d9.into_output());
    let mut en_vcc_geiger = ActiveLowPin::from_pin(pins.d10.into_output());

    en_vcc_1_wire.set_active();
    en_vcc_geiger.set_active();
    en_vcc_i2c.set_active();
    en_vcc_rf.set_active();


    let mut serial = Usart::new(
        dp.USART0, 
        pins.d0, 
        pins.d1.into_output(),
        Baudrate::<MHz8>::new(9600)
    );

    let mut delay = Delay::<MHz8>::new();

    let mut i2c = I2c::with_external_pullup(
        dp.TWI, 
        pins.a4.into_floating_input(), 
        pins.a5.into_floating_input(), 
        50_000
    );

    ufmt::uwrite!(&mut serial, "--------------------------\r\n").unwrap_infallible();

    let mut driver = OokDriver::new(pins.d7.into_output(), MockPin::new());

    let test_data = TypedLabelReadout::<SensorId0, UnitScale1, Thermometer>::new(5);

    loop {
        if driver.is_idle(){
            driver.send(&test_data.get_bytes());
            delay.delay_ms(1000);
        }

        driver.tick();
        delay.delay_ms(10);
    }
}