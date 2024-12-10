//! Pin definitions

use esp_hal::gpio::{GpioPin, Input, Output};
use esp_hal::{analog, peripherals::ADC1};

/// DCC functionality
pub mod dcc {
    use embedded_hal::digital::OutputPin;

    use super::*;
    /// DCC modes
    ///
    /// * [`Operations`] - Normal track operations
    /// * [`Service`] - Service (programming) mode
    ///
    /// Note that this assumes device `ADC1` is used, which may not be the case if pins are changed
    pub trait Mode<'d> {
        type CurrentSense;
        type Enable: OutputPin;
        type Data: OutputPin;
    }

    // Implement modes for dcc types
    impl<'d> Mode<'d> for crate::dcc::Operations {
        type CurrentSense = analog::adc::AdcPin<GpioPin<0>, ADC1>;
        type Enable = Output<'d, GpioPin<1>>;
        type Data = Output<'d, GpioPin<2>>;
    }

    impl<'d> Mode<'d> for crate::dcc::Service {
        type CurrentSense = analog::adc::AdcPin<GpioPin<3>, ADC1>;
        type Enable = Output<'d, GpioPin<4>>;
        type Data = Output<'d, GpioPin<5>>;
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
