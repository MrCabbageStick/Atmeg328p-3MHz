use arduino_hal::hal::delay::Delay;

use crate::local_clock::MHz3;
use core::arch::asm;

pub struct LocalDelay(Delay<MHz3>);

impl LocalDelay{
    pub fn new() -> Self {
        Self(Delay::new())
    }

    pub fn delay_us(&mut self, us: u32) {
        // With 3MHz (3_686_400 Hz) 1 cycle takes ~0,27 us
        // 4 cycles is 1,08 us

        // Function call overhead is 16 cycles (4.3 us)

        if us < 5 { // 3 cycles
            return; // 4 if true
        }

        // Busy loop takes ~1 us (4 cycles)
        // us -= 5; 

        let iters = us >> 12;
        let mut i = 0;

        while i < iters{
            busy_loop(0xfff);
            i += 1;
        }

        busy_loop((us & 0xfff) as u16);
    }

    pub fn delay_ms(&mut self, ms: u32) {
        self.delay_us(ms * 1_000);
    }
}

#[cfg(target_arch = "avr")]
#[allow(unused_assignments)]
/// Assembly loop with n iterations, each taking 4 cycles\
/// Based on https://github.com/arduino/ArduinoCore-avr/blob/master/cores/arduino/wiring.c
fn busy_loop(mut n: u16) {
    unsafe {
        asm!(
            "1:",               // Numeric label 1 
            "sbiw {n}, 1",      // (2 cycles) Subtract 1 from 16-bit register pair
            "brne 1b",          // (2 cycles) If output not 0 (Z registry) go to label 1 backward
            n = inout(reg_iw) n,// Require 16-bit register pair
        );
    }
}