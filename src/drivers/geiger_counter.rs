use core::marker::PhantomData;

use arduino_hal::pac::TC1;

use crate::util::timer::millis;

const MILLIS_IN_SECOND: u32 = 1_000;

pub struct GeigerCounter<'a>{
    last_millis: u32,
    counts: [u16; 60],
    index: usize,
    sum: u32,
    filled: u8,
    timer: &'a TC1,
}

impl<'a> GeigerCounter<'a>{
    pub fn new(tc1: &'a TC1) -> Self{
        Self{ last_millis: 0, index: 0, sum: 0, filled: 0, counts: [0; 60], timer: tc1}
    }

    pub fn init(&self){
        timer1_counter_setup(self.timer);
    }

    pub fn tick(&mut self){
        if millis() - self.last_millis < MILLIS_IN_SECOND {
            return; 
        }

        self.last_millis = millis();

        self.sum -= self.counts[self.index] as u32;

        let count = timer1_read_and_reset(self.timer);

        self.counts[self.index] = count;
        self.sum += count as u32;

        // Advance index, wrapping at 60
        self.index = if self.index == 59 { 0 } else { self.index + 1 };

        if self.filled < 60 {
            self.filled += 1;
        }
    }

    /// Counts per minute
    pub fn cpm(&self) -> u32{
        if self.filled == 0 {
            return 0;
        }

        // Scale to 60 seconds even before buffer is full
        self.sum * 60 / self.filled as u32
    }

    /// How many seconds of data are in the buffer
    pub fn seconds_collected(&self) -> u8 {
        self.filled
    }


}

/// Sets up Timer1 as an external pulse counter on T1 pin (PD5)
fn timer1_counter_setup(tc1: &TC1) {
    unsafe {
        tc1.tccr1a().write(|w| w.wgm1().bits(0b00));

        tc1.tccr1b().write(|w| {
            w.wgm1().bits(0b00)
                .cs1().bits(0b111)
        });

        tc1.tcnt1().write(|w| w.bits(0));
    }
}

fn timer1_read_and_reset(tc1: &TC1) -> u16{
    avr_device::interrupt::free(|_| unsafe {
        let val = tc1.tcnt1().read().bits();
        tc1.tcnt1().write(|w| w.bits(0));
        val
    })
}