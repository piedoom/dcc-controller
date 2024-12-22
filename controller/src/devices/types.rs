use dcc_rs::DccInterruptHandler;
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_hal::{
    Async,
    gpio::{GpioPin, NoPin, Output},
    spi::master::SpiDmaBus,
};
use ssd1322_rs::SSD1322;

use crate::{dcc::Operations, devices::pins};

use super::pins::dcc::Mode;

pub type RotaryEncoder<'d> = rotary_encoder_embedded::RotaryEncoder<
    rotary_encoder_embedded::angular_velocity::AngularVelocityMode,
    pins::rotary_encoder::Data<'d>,
    pins::rotary_encoder::Clock<'d>,
>;

pub type Button<'d, P> = button_driver::Button<P, embassy_time::Instant, embassy_time::Duration>;

pub type LeftButton<'d> = Button<'d, pins::buttons::LeftButton<'d>>;
pub type RightButton<'d> = Button<'d, pins::buttons::RightButton<'d>>;
pub type FnButton<'d> = Button<'d, pins::buttons::FnButton<'d>>;

pub type DccDriver<'d> = DccInterruptHandler<<Operations as Mode<'d>>::Data>;

pub type Display<'d> = SSD1322<
    ExclusiveDevice<SpiDmaBus<'d, Async>, NoPin, NoDelay>,
    Output<'d, GpioPin<22>>,
    Output<'d, GpioPin<19>>,
    NoPin,
>;
