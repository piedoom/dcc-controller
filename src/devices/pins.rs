//! Pin definitions

use esp_hal::gpio::{GpioPin, Input, Output};
use esp_hal::{analog, peripherals::ADC1};

type In<'d, const GPIONUM: u8> = Input<'d, GpioPin<GPIONUM>>;
type Out<'d, const GPIONUM: u8> = Output<'d, GpioPin<GPIONUM>>;

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
        type Enable = Out<'d, 1>;
        type Data = Out<'d, 2>;
    }

    impl<'d> Mode<'d> for crate::dcc::Service {
        type CurrentSense = analog::adc::AdcPin<GpioPin<3>, ADC1>;
        type Enable = Out<'d, 4>;
        type Data = Out<'d, 5>;
    }
}

/// SPI device pins
pub(crate) mod spi {
    use super::*;

    pub(crate) type Dc<'d> = Out<'d, 7>;
    pub(crate) type Sck<'d> = Out<'d, 8>;
    pub(crate) type Res<'d> = Out<'d, 9>;
    pub(crate) type Mosi<'d> = Out<'d, 10>;
}

/// Rotary encoder and switch control
pub(crate) mod rotary_encoder {
    use super::*;

    pub(crate) type Clock<'d> = In<'d, 6>;
    pub(crate) type Data<'d> = In<'d, 20>;
    pub(crate) type Switch<'d> = In<'d, 21>;
}
