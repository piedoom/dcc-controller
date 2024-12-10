use super::*;
use crate::devices::pins;
use dcc_rs::{DccInterruptHandler, packets::SerializeBuffer};
use esp_hal::{analog::adc::Adc, peripherals::ADC1};

/// One ADC device powers both modes. See [`pins`] for more specific definitions
pub(crate) static ADC: Global<Adc<ADC1>> = default();

pub(crate) static ROTARY_ENCODER: Global<types::RotaryEncoder> = default();

pub(crate) static BUTTON: Global<types::Button> = default();

/// Operations mode devices and resources
pub(crate) mod operations {

    use pins::dcc::Mode;

    use crate::dcc::Operations;

    use super::*;

    /// Current sense ADC pin
    pub(crate) static CURRENT_SENSE: Global<<Operations as Mode>::CurrentSense> = default();

    /// Enable DCC pin
    pub(crate) static ENABLE: Global<<Operations as Mode>::Enable> = default();

    /// DCC interrupt handler driver
    pub(crate) static DRIVER: Global<DccInterruptHandler<<Operations as Mode>::Data>> = default();

    /// DCC interrupt timer
    pub(crate) static TIMER: Global<Timer<Timer0<TIMG1>, esp_hal::Blocking>> = default();

    /// Transmission buffer for operations mode
    pub(crate) static TX_BUFFER: Global<(SerializeBuffer, usize)> = default();
}

/// Service (programming) mode devices and resources
pub(crate) mod service {
    use pins::dcc::Mode;

    use crate::dcc::Service;

    use super::*;

    /// Current sense ADC pin
    pub(crate) static CURRENT_SENSE: Global<<Service as Mode>::CurrentSense> = default();

    /// Transmission buffer for service mode
    pub(crate) static TX_BUFFER: Global<(SerializeBuffer, usize)> = default();
}
