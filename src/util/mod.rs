use core::ops::{Div, Rem};

pub mod timer;
pub mod voltage_divider;


pub fn split_fixed_point<T: Div<Output = T> + Rem<Output = T> + Copy>(value: T, n: T) -> (T, T){
    (
        value / n,
        value % n,
    )
}

/// Waits for UASRT to complete sending data
pub fn wait_for_tx_complete() {
    let usart = unsafe { &*arduino_hal::pac::USART0::ptr() };

    while usart.ucsr0a().read().txc0().bit_is_clear() {}
    usart.ucsr0a().modify(|_, w| w.txc0().set_bit());
}