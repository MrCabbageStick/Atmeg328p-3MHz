use arduino_hal::clock::Clock;

pub struct MHz3;

impl Clock for MHz3{
    const FREQ: u32 = 3686400;
}