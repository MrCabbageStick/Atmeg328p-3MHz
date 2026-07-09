use arduino_hal::pac::TC1;

pub struct GeigerCounter<'a>{
    counts: [u16; 60],
    index: usize,
    sum: u32,
    filled: u8,
    timer: &'a TC1,
}

impl<'a> GeigerCounter<'a>{
    pub fn new(tc1: &'a TC1) -> Self{
        Self{index: 0, sum: 0, filled: 0, counts: [0; 60], timer: tc1}
    }

    /// Sets up `self.timer` as an external pulse counter
    pub fn init(&self){
        let tc = self.timer;

        unsafe {
            tc.tccr1a().write(|w| w.wgm1().bits(0b00));

            tc.tccr1b().write(|w| {
                w.wgm1().bits(0b00)
                    .cs1().bits(0b111)
            });

            tc.tcnt1().write(|w| w.bits(0));
        }
    }

    pub fn tick(&mut self){
        self.sum -= self.counts[self.index] as u32;

        let count = self.read_and_reset_timer();

        self.counts[self.index] = count;
        self.sum += count as u32;

        // Advance index, wrapping at 60
        self.index = if self.index == 59 { 0 } else { self.index + 1 };

        if self.filled < 60 {
            self.filled += 1;
        }
    }

    fn read_and_reset_timer(&self) -> u16{
        let tc = self.timer;

        avr_device::interrupt::free(|_| unsafe {
            let val = tc.tcnt1().read().bits();
            tc.tcnt1().write(|w| w.bits(0));
            val
        })
    }

    /// Counts per minute
    pub fn cpm(&self) -> u32{
        // Scale to 60 seconds even before buffer is full
        self.sum * 60 / self.filled as u32
    }

    /// How many seconds of data are in the buffer
    pub fn seconds_collected(&self) -> u8 {
        self.filled
    }
}