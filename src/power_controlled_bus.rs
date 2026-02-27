use arduino_hal::port::{Pin, PinOps, mode::Output};

/// Active low pin for driving pulled high P-MOSFETS
pub struct ActiveLowPin<PIN>{
    pin: Pin<Output, PIN>
}

impl<PIN> ActiveLowPin<PIN> where PIN: PinOps{
    pub fn from_pin(pin: Pin<Output, PIN>) -> Self {
        Self { pin }
    }

    pub fn set_active(&mut self){
        self.pin.set_low();
    }

    pub fn set_notactive(&mut self){
        self.pin.set_high();
    }

    pub fn toggle(&mut self){
        self.pin.toggle();
    }
}