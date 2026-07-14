use core::cell::Cell;

use arduino_hal::pac::TC2;
use avr_device::interrupt::{self, Mutex};

/// Determines if timer2 interrupt occurred
static TIMER2_FLAG: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

/// Returns true if at least 1 interrupt occurred
pub fn should_transmitter_tick() -> bool {
    let mut flag = false;

    // Read and clear `TIMER2_FLAG`
    interrupt::free(|cs|{
        flag = TIMER2_FLAG.borrow(cs).get();
        TIMER2_FLAG.borrow(cs).set(false);
    });

    flag
}

pub fn setup_radio_timer(tc2: &TC2){
    // Normal port operation, CTC
    tc2.tccr2a().write(|w| w.wgm2().ctc());

    // Prescaler
    tc2.tccr2b().write(|w| w.cs2().prescale_8());

    unsafe{
        // Value to compare counter with
        tc2.ocr2a().write(|w| w.bits(64));
    }

    // Enable timer2 interrupt
    tc2.timsk2().write(|w| w.ocie2a().set_bit());
}

#[avr_device::interrupt(atmega328p)]
fn TIMER2_COMPA(){
    // Set `TIMER2_FLAG` to true
    interrupt::free(|cs| {
        let flag = TIMER2_FLAG.borrow(cs);
        flag.set(true);
    });
}