#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use arduino_hal::{Adc, I2c, Peripherals, hal::{delay::Delay}, pac::{CPU, WDT}};
use embedded_hal::delay::DelayNs;
use ook_433mhz::driver::transmitter::Transmitter;
// use panic_halt as _;
use arduino_hal::hal::clock::MHz8;

use battery_free_climat_sensor::{climate_sensor::ClimateSensor, power_manager::PowerManager, radio_timer::{setup_radio_timer, should_transmitter_tick}, sleep::{self}};
use battery_free_climat_sensor::drivers::{bmp280::config::DefaultConfig as Bmp280DefaultConf, veml7700::config::ConfigFastLowPower as Veml7700DefaultConf};

/// How many times transmitter will send data end-to-end
const SEND_DATA_N_TIMES: u8 = 3;
/// Frequency of atmega oscillator
type OscillatorFreq = MHz8;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    // Prepare adc for reading capacitor voltage
    let mut adc = Adc::new(dp.ADC, Default::default());

    // Modules power management, with active low pins
    let mut power_manager = PowerManager::new(
        pins.d6.into_output(), 
        pins.d10.into_output(), 
        pins.d8.into_output(), 
        true
    );
    power_manager.deactivate_all();

    // 1-wire vcc pin is used as status LED pin, 
    // as 1-wire is not implemented yet
    let mut status_led = pins.d9.into_output_high();

    // Prepare i2c bus
    let i2c = I2c::with_external_pullup(
        dp.TWI, 
        pins.a4.into_floating_input(), 
        pins.a5.into_floating_input(), 
        50_000
    );

    // Set up transmitter on pin d7
    let mut transmitter = Transmitter::<8, _>::new(pins.d7.into_output());

    // Prepare climate sensor with id of 1,
    // pin a1 to monitor the summed voltage on
    // capacitors, and pin a0 for second
    // capacitor
    let mut climate_sensor = ClimateSensor::<
        Veml7700DefaultConf,
        Bmp280DefaultConf, _, _, _
    >::new(
        1,
        i2c,
        Delay::<OscillatorFreq>::new(),
        pins.a1.into_analog_input(&mut adc),
        pins.a0.into_analog_input(&mut adc),
    );

    // Setup timers
    sleep::setup_wdt(&dp.CPU, &dp.WDT);
    setup_radio_timer(&dp.TC2);
    // Enable interrupts
    unsafe { avr_device::interrupt::enable(); }

    // Create delay instance 
    let mut delay = Delay::<OscillatorFreq>::new();

    // Activate all devices
    power_manager.activate_all();

    // Climate sensor init flag
    let mut initialized = false;

    loop {
        // Sleep
        while !sleep::ready(){
            sleep::enter_sleep(&dp.CPU);
        }

        // Activate LED to show that climate
        // sensor is working
        status_led.set_low();
        // Activate devices that were disabled
        // for duration of the sleep 
        power_manager.activate_power_hungry();

        // Give devices time to power-up
        delay.delay_ms(50);

        // If climate sensor not initialized, initialize it
        if !initialized{
            match climate_sensor.init(){
                Ok(_) => {
                    initialized = true;
                }
                Err(_err) => {
                    initialized = false;
                    continue;
                }
            }
        }

        // Read bytes from climate sensor
        match climate_sensor.read_bytes(&mut adc){
            Ok(data) => {
                // Send data a few times
                for _ in 0..SEND_DATA_N_TIMES {
                    transmitter.send(&data);

                    // Tick the transmitter 
                    while !transmitter.is_idle(){
                        if should_transmitter_tick(){
                            transmitter.transmit();
                        }
                    }
                }
            },
            // Skip errors
            Err(_err) => {}
        }

        // Disable power hungry devices
        power_manager.deactivate_power_hungry();
        // Turn off the status LED
        status_led.set_high();
    }
}


use core::panic::{PanicInfo};
use avr_device::asm::nop;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let dp = unsafe {
        Peripherals::steal()
    };

    let pins = arduino_hal::pins!(dp);

    // Set pin 9 as debug LED
    let mut led = pins.d9.into_output_high();
    // Light up the debug LED
    led.set_low();

    // Force reset via watchdog
    set_wdt_for_reset(&dp.CPU, &dp.WDT);

    // Blink the LED to indicate unhandled panic
    loop {
        led.toggle();
        for _ in 0..50_000{ nop(); }
    }
}

/// Set watchdog to reset mode with the shortest 
/// time available, to force atmega restart
fn set_wdt_for_reset(cpu: &CPU, wdt: &WDT){
    avr_device::interrupt::free(|_|{
        avr_device::asm::wdr();
        cpu.mcusr().modify(|_, w|unsafe{ w.bits(0)});

        // Start timed sequence: set WDCE + WDE, preserving nothing else
        // needed since WDTCSR resets to 0 anyway.
        wdt.wdtcsr().write(|w| unsafe { w.bits(0b0001_1000) }); // WDCE | WDE

        // Must land within 4 clock cycles of the write above:
        // WDE=1, WDP=000 -> 16ms system reset mode, WDIE=0
        wdt.wdtcsr().write(|w| unsafe { w.bits(0b0000_1000) }); // WDE only
    });
}