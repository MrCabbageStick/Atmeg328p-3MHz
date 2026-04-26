#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_hal::{I2c, hal::{delay::Delay, usart::Usart}, prelude::{_unwrap_infallible_UnwrapInfallible}, usart::Baudrate};
use embedded_hal::delay::DelayNs;
use panic_halt as _;
use arduino_hal::hal::clock::MHz8;

use battery_free_climat_sensor::{drivers::{bmp280::{config::DefaultConfig, driver::Bmp280}, veml7700::{config::ConfigFastLowPower, driver::Veml7700}}, power_controlled_bus::ActiveLowPin};

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

    // let veml7700 = Veml7700::<ConfigFastLowPower>::new(0x10);
    
    // match veml7700.init(&mut i2c){
    //     Ok(_) => ufmt::uwrite!(&mut serial, "VEML7700 initialized\r\n").unwrap_infallible(),
    //     Err(e) => ufmt::uwrite!(&mut serial, "Unable to initialize VEML7700: \n{:?}\r\n", e).unwrap_infallible()
    // }

    // delay.delay_ms(100);

    let mut bmp280 = Bmp280::<DefaultConfig>::new(0x77);

    match bmp280.init(&mut i2c){
        Ok(_) => ufmt::uwrite!(&mut serial, "Bmp280 initilized\r\n").unwrap_infallible(),
        Err(e) => ufmt::uwrite!(&mut serial, "Unable to initialize Bmp280: {:?}\r\n", e).unwrap_infallible(),
    };

    delay.delay_ms(100);

    loop {
        
        // match veml7700.read(&mut i2c){
        //     Ok(lx) => ufmt::uwrite!(&mut serial, "Light sensor: {} lx\r\n", lx).unwrap_infallible(),
        //     Err(_) => ufmt::uwrite!(&mut serial, "Unable to read light sensor data\r\n").unwrap_infallible()
        // }

        match bmp280.read(&mut i2c){
            Ok(data) => ufmt::uwrite!(
                &mut serial,
                "BMP280 Data:\n\r- Temp: {}°mC\r\n- Pressure: {} Pa\r\n", 
                data.temperature,
                data.pressure
            ).unwrap_infallible(),
            Err(e) => ufmt::uwrite!(&mut serial, "Unable to read BMP280 data: {:?}\r\n", e).unwrap_infallible(),
        }
        
        delay.delay_ms(1000);
    }
}