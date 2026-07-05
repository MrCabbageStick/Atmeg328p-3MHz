#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_hal::{Adc, I2c, hal::{delay::Delay, usart::Usart}, prelude::{_unwrap_infallible_UnwrapInfallible}, usart::Baudrate};
use ook_433mhz::driver::transmitter::Transmitter;
use panic_halt as _;
use arduino_hal::hal::clock::MHz8;

use battery_free_climat_sensor::{climate_sensor::ClimateSensor, power_manager::PowerManager, radio::{setup_timer_2, should_transmitter_tick}, sleep::{self}, util};
use battery_free_climat_sensor::drivers::{bmp280::config::DefaultConfig as Bmp280DefaultConf, veml7700::config::ConfigFastLowPower as Veml7700DefaultConf};


const SEND_DATA_N_TIMES: u8 = 2;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut adc = Adc::new(dp.ADC, Default::default());

    // Modules power management
    let mut power_manager = PowerManager::new(pins.d6.into_output(), pins.d10.into_output(), pins.d8.into_output(), true);
    power_manager.deactivate_all();

    // 1-wire vcc pin is used as status LED pin, as 1-wire is not implemented
    let mut status_led = pins.d9.into_output_high();

    // Communication
    let mut serial = Usart::new(
        dp.USART0, 
        pins.d0, 
        pins.d1.into_output(),
        Baudrate::<MHz8>::new(9600)
    );

    let i2c = I2c::with_external_pullup(
        dp.TWI, 
        pins.a4.into_floating_input(), 
        pins.a5.into_floating_input(), 
        50_000
    );

    sleep::setup_wdt(&dp.CPU, &dp.WDT);
    setup_timer_2(&dp.TC2);
    unsafe { avr_device::interrupt::enable(); }

    let mut transmitter = Transmitter::<5, _>::new(pins.d7.into_output());

    let mut climate_sensor = ClimateSensor::<
        Veml7700DefaultConf,
        Bmp280DefaultConf, _, _, _
    >::new(
        1,
        i2c,
        Delay::<MHz8>::new(),
        pins.a1.into_analog_input(&mut adc),
        pins.a0.into_analog_input(&mut adc),
    );

    power_manager.activate_all();

    ufmt::uwrite!(&mut serial, "[INFO]: Powered on\r\n").unwrap_infallible();
    util::wait_for_tx_complete();

    let mut initialized = false;

    loop {
        sleep::enter_sleep(&dp.CPU);

        if !sleep::ready(){
            continue;
        }

        status_led.set_low();
        power_manager.activate_power_hungry();

        // If climate sensor not initialized, initialize it
        if !initialized{
            match climate_sensor.init(){
                Ok(_) => {
                    ufmt::uwrite!(&mut serial, "[INFO]: Climate sensor initilized\r\n").unwrap_infallible();
                    initialized = true;
                }
                Err(err) => {
                    ufmt::uwrite!(&mut serial, "[ERROR]: Climate sensor fialed to initialize. Err: {:?}\r\n", err).unwrap_infallible();
                    initialized = false;
                    continue;
                }
            }
        }

        match climate_sensor.read_bytes(&mut adc){
            Ok(data) => {
                ufmt::uwrite!(&mut serial, "[INFO]: New readout\r\n").unwrap_infallible();

                for _ in 0..SEND_DATA_N_TIMES {
                    transmitter.send(&data);

                    while !transmitter.is_idle(){
                        if should_transmitter_tick(){
                            transmitter.transmit();
                        }
                    }
                }
            },
            Err(err) => {
                ufmt::uwrite!(&mut serial, "[ERROR]: Error while reading the data: {:?}\r\n", err).unwrap_infallible()
            }
        }

        util::wait_for_tx_complete(); 

        power_manager.deactivate_power_hungry();
        status_led.set_high();
    }
}