//! Devices and resources used throughout the application

use core::cell::RefCell;

use critical_section::Mutex;
use esp_hal::{
    Blocking,
    gpio::{GpioPin, Output},
    peripherals::{SPI2, TIMG1},
    spi::master::Spi,
    timer::timg::{Timer, Timer0},
};

pub mod dcc;
pub mod pins;
pub mod types;

/// Default initialization for resources. After initialized at the start of the program, all should be assumed to have a `Some` value
const fn default<T>() -> Global<T> {
    Mutex::new(RefCell::new(Option::<T>::None))
}

pub(crate) type Global<T> = Mutex<RefCell<Option<T>>>;

/// SPI display device
pub(crate) static DISPLAY: Global<
    ssd1331::Ssd1331<Spi<'_, Blocking, SPI2>, Output<'_, GpioPin<7>>>,
> = default();
