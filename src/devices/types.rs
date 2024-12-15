use esp_hal::{
    Blocking,
    gpio::{GpioPin, Output},
    peripherals::SPI2,
    spi::master::Spi,
};

use crate::devices::pins;

pub type RotaryEncoder<'d> = rotary_encoder_embedded::RotaryEncoder<
    rotary_encoder_embedded::angular_velocity::AngularVelocityMode,
    pins::rotary_encoder::Data<'d>,
    pins::rotary_encoder::Clock<'d>,
>;

pub type Button<'d> = button_driver::Button<
    pins::rotary_encoder::Switch<'d>,
    embassy_time::Instant,
    embassy_time::Duration,
>;

pub type Display<'d> = ssd1331::Ssd1331<Spi<'d, Blocking, SPI2>, Output<'d, GpioPin<7>>>;
