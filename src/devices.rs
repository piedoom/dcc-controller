//! Devices and resources used throughout the application

use core::cell::RefCell;

use critical_section::Mutex;
use esp_hal::{
    Blocking,
    peripherals::TIMG1,
    prelude::*,
    spi::master::Spi,
    timer::timg::{Timer, Timer0},
};

/// Default initialization for resources. After initialized at the start of the program, all should be assumed to have a `Some` value
const fn default<T>() -> Global<T> {
    Mutex::new(RefCell::new(Option::<T>::None))
}

pub(crate) type Global<T> = Mutex<RefCell<Option<T>>>;

/// Noop the Cs pin since we don't need it
pub(crate) struct NoCs;
impl embedded_hal::digital::OutputPin for NoCs {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl embedded_hal::digital::ErrorType for NoCs {
    type Error = core::convert::Infallible;
}

pub(crate) mod types {

    use embassy_time::{Duration, Instant};

    use super::*;
    use crate::pins;

    pub(crate) type RotaryEncoder<'d> = rotary_encoder_embedded::RotaryEncoder<
        rotary_encoder_embedded::angular_velocity::AngularVelocityMode,
        pins::rotary_encoder::Data<'d>,
        pins::rotary_encoder::Clock<'d>,
    >;

    pub(crate) type Button<'d> =
        button_driver::Button<pins::rotary_encoder::Switch<'d>, Instant, Duration>;
}

// /// SPI display device
// pub(crate) static DISPLAY: Global<
//     mipidsi::Display<
//         display_interface_spi::SPIInterface<
//             embedded_hal_bus::spi::ExclusiveDevice<
//                 Spi<'_, Blocking, esp_hal::peripherals::SPI2>,
//                 NoCs,
//                 embedded_hal_bus::spi::NoDelay,
//             >,
//             crate::pins::spi::Dc,
//         >,
//         mipidsi::models::ST7789,
//         mipidsi::NoResetPin,
//     >,
// > = default();

pub(crate) mod dcc {
    use crate::pins;

    use super::*;
    use button_driver::Button;
    use dcc_rs::{DccInterruptHandler, packets::SerializeBuffer};
    use embassy_time::{Duration, Instant};
    use esp_hal::{analog::adc::Adc, gpio::Input, peripherals::ADC1};
    use pins::rotary_encoder;
    use rotary_encoder_embedded::{RotaryEncoder, angular_velocity::AngularVelocityMode};

    /// One ADC device powers both modes. See [`pins`] for more specific definitions
    pub(crate) static ADC: Global<Adc<ADC1>> = default();

    pub(crate) static ROTARY_ENCODER: Global<types::RotaryEncoder> = default();

    pub(crate) static BUTTON: Global<types::Button> = default();

    /// Operations mode devices and resources
    pub(crate) mod operations {
        use crate::pins::dcc::operations::*;

        use super::*;

        /// Current sense ADC pin
        pub(crate) static CURRENT_SENSE: Global<CurrentSense> = default();

        /// Enable DCC pin
        pub(crate) static ENABLE: Global<Enable> = default();

        /// DCC interrupt handler driver
        pub(crate) static DRIVER: Global<DccInterruptHandler<Data>> = default();

        /// DCC interrupt timer
        pub(crate) static TIMER: Global<Timer<Timer0<TIMG1>, esp_hal::Blocking>> = default();

        /// Transmission buffer for operations mode
        pub(crate) static TX_BUFFER: Global<(SerializeBuffer, usize)> = default();
    }

    /// Service (programming) mode devices and resources
    pub(crate) mod service {
        use super::*;
        use crate::pins::dcc::service::*;

        /// Current sense ADC pin
        pub(crate) static CURRENT_SENSE: Global<CurrentSense> = default();

        /// Transmission buffer for service mode
        pub(crate) static TX_BUFFER: Global<(SerializeBuffer, usize)> = default();
    }
}
