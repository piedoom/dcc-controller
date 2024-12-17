use dcc_rs::DccInterruptHandler;
use esp_hal::{
    Blocking,
    gpio::{GpioPin, Output},
    peripherals::SPI2,
    spi::master::Spi,
};

use crate::{dcc::Operations, devices::pins};

use super::pins::dcc::Mode;

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

pub type DccDriver<'d> = DccInterruptHandler<<Operations as Mode<'d>>::Data>;

pub type Display<'d> = ssd1331::Ssd1331<Spi<'d, Blocking, SPI2>, Output<'d, GpioPin<7>>>;
