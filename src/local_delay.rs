use arduino_hal::hal::delay::Delay;
use crate::local_clock::MHz3;
use core::arch::asm;

pub struct LocalDelay(Delay<MHz3>);

impl LocalDelay {
    pub fn new() -> Self {
        Self(Delay::new())
    }

    pub fn delay_us(&mut self, us: u32) {
        if us == 0 { return; }

        // Cycles per microsecond = 3.6864
        // Our loop below takes exactly 4 cycles per iteration.
        // Target iterations = us * (3.6864 / 4) = us * 0.9216
        
        // Fixed-point math (236/256 ≈ 0.9218)
        // This math is fast on AVR and very close to the target ratio.
        let ticks = (us * 189) >> 8;

        if ticks > 0 {
            busy_loop_24(ticks);
        }
    }

    pub fn delay_ms(&mut self, ms: u32) {
        for _ in 0..ms {
            self.delay_us(1000);
        }
    }
}

#[cfg(target_arch = "avr")]
/// A 24-bit busy loop (supports delays up to ~18 seconds at 3.6MHz)
/// Each iteration takes exactly 4 cycles.
fn busy_loop_24(mut n: u32) {
    unsafe {
        asm!(
            "1:",
            "subi {0}, 1",      // 1 cycle: Subtract from LSB
            "sbci {1}, 0",      // 1 cycle: Subtract carry from mid-byte
            "sbci {2}, 0",      // 1 cycle: Subtract carry from MSB
            "brne 1b",          // 2 cycles (back to 1) or 1 cycle (exit)
            inout(reg) n as u8 => _, 
            inout(reg) (n >> 8) as u8 => _,
            inout(reg) (n >> 16) as u8 => _,
        );
    }
}

#[cfg(not(target_arch = "avr"))]
fn busy_loop_24(_n: u32) {}