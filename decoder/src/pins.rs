use esp_hal::gpio::{GpioPin, Input};

pub type Data<'d> = Input<'d, GpioPin<5>>;
