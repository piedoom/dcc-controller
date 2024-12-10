use embassy_time::{Duration, Instant};

use crate::devices::pins;

pub(crate) type RotaryEncoder<'d> = rotary_encoder_embedded::RotaryEncoder<
    rotary_encoder_embedded::angular_velocity::AngularVelocityMode,
    pins::rotary_encoder::Data<'d>,
    pins::rotary_encoder::Clock<'d>,
>;

pub(crate) type Button<'d> =
    button_driver::Button<pins::rotary_encoder::Switch<'d>, Instant, Duration>;
