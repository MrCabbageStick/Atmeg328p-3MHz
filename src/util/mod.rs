use core::ops::{Div, Rem};

pub mod timer;


pub fn split_fixed_point<T: Div<Output = T> + Rem<Output = T> + Copy>(value: T, n: T) -> (T, T){
    (
        value / n,
        value % n,
    )
}