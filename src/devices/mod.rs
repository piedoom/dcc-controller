//! Devices and resources used throughout the application

use core::cell::RefCell;

use critical_section::Mutex;
use esp_hal::{
    peripherals::TIMG1,
    timer::timg::{Timer, Timer0},
};

pub mod dcc;
pub mod pins;
pub mod types;

/// Default initialization for resources. After initialized at the start of the program, all should be assumed to have a `Some` value
pub(crate) const fn default<T>() -> Global<T> {
    Mutex::new(RefCell::new(Option::<T>::None))
}

pub(crate) type Global<T> = Mutex<RefCell<Option<T>>>;
