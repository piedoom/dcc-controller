//! Pin definitions

use esp_hal::gpio::{GpioPin, Input, Output};

/// DCC functionality
pub(crate) mod dcc {
    use super::*;

    /// Normal operations mode pins
    pub(crate) mod operations {
        use super::*;

        use esp_hal::{analog::adc::AdcPin, peripherals::ADC1};

        pub(crate) type CurrentSense = AdcPin<GpioPin<0>, ADC1>;
        pub(crate) type Enable<'d> = Output<'d, GpioPin<1>>;
        pub(crate) type Data<'d> = Output<'d, GpioPin<2>>;
    }

    /// Programming (service) mode pins
    pub(crate) mod service {
        use super::*;

        use esp_hal::{analog::adc::AdcPin, peripherals::ADC1};

        pub(crate) type CurrentSense = AdcPin<GpioPin<3>, ADC1>;
        pub(crate) type Enable<'d> = Output<'d, GpioPin<4>>;
        pub(crate) type Data<'d> = Output<'d, GpioPin<5>>;
    }
}

/// SPI device pins
pub(crate) mod spi {
    use super::*;

    pub(crate) type Mosi<'d> = Output<'d, GpioPin<10>>;
    pub(crate) type Sck<'d> = Output<'d, GpioPin<8>>;
    pub(crate) type Dc<'d> = Output<'d, GpioPin<7>>;
    pub(crate) type Res<'d> = Output<'d, GpioPin<9>>;
}

/// Rotary encoder and switch control
pub(crate) mod rotary_encoder {
    use super::*;

    pub(crate) type Data<'d> = Input<'d, GpioPin<20>>;
    pub(crate) type Clock<'d> = Input<'d, GpioPin<6>>;
    pub(crate) type Switch<'d> = Input<'d, GpioPin<21>>;
}
