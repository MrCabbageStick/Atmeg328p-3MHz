use core::ops::{Div, Rem};

pub mod timer;
pub mod voltage_divider;

/// Returns a tuple of (whole number, fraction),
/// where fraction is determined by `n`
pub fn split_fixed_point<T: Div<Output = T> + Rem<Output = T> + Copy>(value: T, n: T) -> (T, T){
    (
        value / n,
        value % n,
    )
}

/// Waits for USART to complete sending data
pub fn wait_for_tx_complete() {
    let usart = unsafe { &*arduino_hal::pac::USART0::ptr() };

    // Wait for `txc0` bit to clear
    while usart.ucsr0a().read().txc0().bit_is_clear() {}
    // Set bit back to one
    usart.ucsr0a().modify(|_, w| w.txc0().set_bit());
}