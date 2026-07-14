use core::cell::Cell;

use arduino_hal::pac::{CPU, WDT};
use avr_device::interrupt;

// Sleep for ~1 minute
const WAKE_CYCLES: u8 = 8;
// Counts how many times watchdog waked up 
// the microcontroller. By default set to 
// `WAKE_CYCLES` to not sleep on first
// call to `ready()` function
static WDT_COUNT: interrupt::Mutex<Cell<u8>> = interrupt::Mutex::new(Cell::new(WAKE_CYCLES));

/// Returns true when `WDT_COUNT` reaches `WAKE_CYCLES`
/// and the reset it
pub fn ready() -> bool {
    interrupt::free(|cs|{
        let cell = WDT_COUNT.borrow(cs);
        let count = cell.get();
        if count >= WAKE_CYCLES {
            cell.set(0);
            true
        } else {
            false
        }
    })
}

pub fn enable_peripherals(cpu: &CPU){
    cpu.prr().write(|w| unsafe { w.bits(0x00) });
}

/// Shut down peripherals via Power Reduction Register
pub fn disable_peripherals(cpu: &CPU) {
    // Power Reduction Register: shut off timers, USART, SPI, TWI.
    // Keep ADC on
    cpu.prr().write(|w| unsafe { w.bits(0b11111110) });
}

/// Sets up Watchdog Timer to interrupt and not reset
/// with ~8s intervals
pub fn setup_wdt(cpu: &CPU, wdt: &WDT) {
    interrupt::free(|_cs| {

        // Reset WDT and clear WDRF in MCUSR first
        avr_device::asm::wdr();
        // Clear system reset flag
        cpu.mcusr().modify(|_, w| w.wdrf().clear_bit());

        // Enable configuration change, then set WDIE (interrupt mode) + ~8s prescaler
        wdt.wdtcsr().modify(|_, w| w.wdce().set_bit().wde().set_bit());
        wdt.wdtcsr().write(|w| unsafe {
            w.wdie().set_bit();   // interrupt mode, not reset mode

            w.wdpl().bits(0b001); // WDP0:2
            w.wdph().bit(true)    // WDP3  -> together ~8.0s timeout
        });
    });
}

/// Puts microcontroller to sleep
pub fn enter_sleep(cpu: &CPU) {
    disable_peripherals(cpu);

    // Set sleep mode to `power down` and set `sleep enable` bit
    cpu.smcr().write(|w| w.sm().pdown().se().set_bit());

    // BOD disable: timing-critical, needs interrupts off only for this part
    interrupt::free(|_cs| {
        cpu.mcucr().modify(|_, w| w.bods().set_bit().bodse().set_bit());
        cpu.mcucr().modify(|_, w| w.bods().set_bit().bodse().clear_bit());
    });

    // Enable interrupts in case not already enabled
    unsafe {
        avr_device::interrupt::enable();
    }
    // Go to sleep
    avr_device::asm::sleep();

    // Clear `sleep enable` bit
    cpu.smcr().modify(|_, w| w.se().clear_bit());
    // Enable all peripherals
    enable_peripherals(cpu);
}

#[avr_device::interrupt(atmega328p)]
fn WDT() {
    // Increment `WDT_COUNT`
    interrupt::free(|cs| {
        let cell = WDT_COUNT.borrow(cs);

        let value = cell.get();
        
        if value < u8::MAX{
            cell.set(value + 1);
        }
    });

    // Re-arm interrupt mode - hardware clears WDIE after firing.
    unsafe {
        let wdt = &(*avr_device::atmega328p::WDT::ptr());
        wdt.wdtcsr().modify(|_, w| w.wdie().set_bit());
    }
}