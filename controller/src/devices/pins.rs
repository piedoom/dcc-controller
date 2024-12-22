//! Pin definitions

use esp_hal::gpio::{GpioPin, Input, Output};
use esp_hal::{analog, peripherals::ADC1};

pub const OPERATIONS_CURRENT_SENSE: u8 = 4;
pub const OPERATIONS_ENABLE: u8 = 2;
pub const OPERATIONS_DATA: u8 = 3;

pub const SERVICE_CURRENT_SENSE: u8 = 5;
pub const SERVICE_ENABLE: u8 = 15;
pub const SERVICE_DATA: u8 = 23;

pub const SPI_DC: u8 = 22;
pub const SPI_SCK: u8 = 21;
pub const SPI_RES: u8 = 19;
pub const SPI_MOSI: u8 = 20;

pub const ROTARY_CLK: u8 = 12;
pub const ROTARY_DATA: u8 = 13;
pub const ROTARY_SW: u8 = 11;

pub const LEFT_BUTTON: u8 = 1;
pub const RIGHT_BUTTON: u8 = 10;
pub const FN_BUTTON: u8 = 11;

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
        type CurrentSense = analog::adc::AdcPin<GpioPin<OPERATIONS_CURRENT_SENSE>, ADC1>;
        type Enable = Out<'d, OPERATIONS_ENABLE>;
        type Data = Out<'d, OPERATIONS_DATA>;
    }

    impl<'d> Mode<'d> for crate::dcc::Service {
        type CurrentSense = analog::adc::AdcPin<GpioPin<SERVICE_CURRENT_SENSE>, ADC1>;
        type Enable = Out<'d, SERVICE_ENABLE>;
        type Data = Out<'d, SERVICE_DATA>;
    }
}

/// SPI device pins
pub(crate) mod spi {
    use super::*;

    pub(crate) type Dc<'d> = Out<'d, SPI_DC>;
    pub(crate) type Sck<'d> = Out<'d, SPI_SCK>;
    pub(crate) type Res<'d> = Out<'d, SPI_RES>;
    pub(crate) type Mosi<'d> = Out<'d, SPI_MOSI>;
}

/// Rotary encoder and switch control
pub(crate) mod rotary_encoder {
    use super::*;

    pub(crate) type Clock<'d> = In<'d, ROTARY_CLK>;
    pub(crate) type Data<'d> = In<'d, ROTARY_DATA>;
}

pub(crate) mod buttons {
    use super::*;
    pub(crate) type LeftButton<'d> = In<'d, LEFT_BUTTON>;
    pub(crate) type RightButton<'d> = In<'d, RIGHT_BUTTON>;
    pub(crate) type FnButton<'d> = In<'d, FN_BUTTON>;
}
