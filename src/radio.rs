use core::cell::Cell;

use arduino_hal::pac::TC2;
use avr_device::interrupt::{self, Mutex};

static mut TIMER2_FLAG: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

pub fn should_tick() -> bool {
    let mut flag = false;

    unsafe{
            interrupt::free(|cs|{
            flag = TIMER2_FLAG.borrow(cs).get();
            TIMER2_FLAG.borrow(cs).set(false);
        });
    }

    flag
}

pub fn setup_timer_2(tc2: &TC2){
    // Normal port operation, CTC
    tc2.tccr2a().write(|w| w.wgm2().ctc());

    // Prescaler
    tc2.tccr2b().write(|w| w.cs2().prescale_128());

    unsafe{
        tc2.ocr2a().write(|w| w.bits(127));
    }

    tc2.timsk2().write(|w| w.ocie2a().set_bit());
}

#[avr_device::interrupt(atmega328p)]
fn TIMER2_COMPA(){
    interrupt::free(|cs| {
        unsafe{
            let flag = TIMER2_FLAG.borrow(cs);
            flag.set(true);
        }
    });
}