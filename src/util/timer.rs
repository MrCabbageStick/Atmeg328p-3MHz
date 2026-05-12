use core::cell::Cell;
use avr_device::interrupt::{self, Mutex};
use arduino_hal::pac::TC0;

// Global millisecond counter
static MILLIS_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

/// Call once at startup to configure Timer0 for millis tracking
pub fn millis_init(tc0: TC0) {
    // Normal port operation, WGM = CTC (mode 2 — clear on compare match)
    tc0.tccr0a().write(|w| w.wgm0().ctc());

    // Prescaler 64
    tc0.tccr0b().write(|w| w.cs0().prescale_64());

    // Compare match value: triggers every 1ms
    // 8MHz / 64 / 1000 - 1 = 124
    unsafe {
        tc0.ocr0a().write(|w| w.bits(124));
    }

    // Enable compare match A interrupt
    tc0.timsk0().write(|w| w.ocie0a().set_bit());

    // Enable global interrupts
    unsafe { interrupt::enable() };
}

/// Returns milliseconds elapsed since millis_init() was called
pub fn millis() -> u32 {
    interrupt::free(|cs| MILLIS_COUNTER.borrow(cs).get())
}


#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    // Timer0 compare match A interrupt — fires every 1ms
    interrupt::free(|cs| {
        let counter = MILLIS_COUNTER.borrow(cs);
        counter.set(counter.get() + 1);
    });
}