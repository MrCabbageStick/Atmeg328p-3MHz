use arduino_hal::port::{Pin, PinOps, mode::Output};
use embedded_hal::digital::{OutputPin, PinState};

pub struct PowerManager<GeigerPin, I2cPin, RfPin>{
    rf_vcc_pin: Pin<Output, RfPin>,
    geiger_vcc_pin: Pin<Output, GeigerPin>,
    i2c_vcc_pin: Pin<Output, I2cPin>,
    active_low: bool,
}

impl<GeigerPin, I2cPin, RfPin> PowerManager<GeigerPin, I2cPin, RfPin>
where GeigerPin: PinOps, I2cPin: PinOps, RfPin: PinOps{
    pub fn new(
        rf_vcc_pin: Pin<Output, RfPin>,
        geiger_vcc_pin: Pin<Output, GeigerPin>,
        i2c_vcc_pin: Pin<Output, I2cPin>,
        active_low: bool,
    ) -> Self{
        Self{rf_vcc_pin, geiger_vcc_pin, i2c_vcc_pin, active_low}
    }

    pub fn deactivate_power_hungry(&mut self){
        let state = (false ^ self.active_low).into();
        let _ = self.rf_vcc_pin.set_state(state);
    }

    pub fn activate_power_hungry(&mut self){
        let state = (true ^ self.active_low).into();
        let _ = self.rf_vcc_pin.set_state(state);
    }

    pub fn deactivate_all(&mut self){
        self.set_all_state((false ^ self.active_low).into());
    }

    pub fn activate_all(&mut self){
        self.set_all_state((true ^ self.active_low).into());
    }

    fn set_all_state(&mut self, state: PinState){
        let _ = self.rf_vcc_pin.set_state(state);
        let _ = self.geiger_vcc_pin.set_state(state);
        let _ = self.i2c_vcc_pin.set_state(state);
    }
}