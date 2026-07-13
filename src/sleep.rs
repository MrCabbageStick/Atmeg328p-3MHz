use core::cell::Cell;

use arduino_hal::pac::{CPU, WDT};
use avr_device::interrupt;

// Sleep for ~7,5 minutes
const WAKE_CYCLES: u8 = 8;
static WDT_COUNT: interrupt::Mutex<Cell<u8>> = interrupt::Mutex::new(Cell::new(WAKE_CYCLES));


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

pub fn disable_peripherals(cpu: &CPU) {
    // Power Reduction Register: shut off timers, USART, SPI, TWI, ADC clock.
    cpu.prr().write(|w| unsafe { w.bits(0b11111110) });
}

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

pub fn enter_sleep(cpu: &CPU) {
    disable_peripherals(cpu);

    cpu.smcr().write(|w| w.sm().pdown().se().set_bit());

    // BOD disable: timing-critical, needs interrupts off only for this part
    interrupt::free(|_cs| {
        cpu.mcucr().modify(|_, w| w.bods().set_bit().bodse().set_bit());
        cpu.mcucr().modify(|_, w| w.bods().set_bit().bodse().clear_bit());
    });

    unsafe {
        avr_device::interrupt::enable();
    }
    avr_device::asm::sleep();

    cpu.smcr().modify(|_, w| w.se().clear_bit());
    enable_peripherals(cpu);
}

#[avr_device::interrupt(atmega328p)]
fn WDT() {
    interrupt::free(|cs| {
        let cell = WDT_COUNT.borrow(cs);
        cell.set(cell.get() + 1);
    });

    // Re-arm interrupt mode — hardware clears WDIE after firing.
    unsafe {
        let wdt = &(*avr_device::atmega328p::WDT::ptr());
        wdt.wdtcsr().modify(|_, w| w.wdie().set_bit());
    }
}