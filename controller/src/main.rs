#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(associated_type_defaults)]
#![feature(async_closure)]

mod dcc;
pub mod devices;
pub mod tasks;

use button_driver::ButtonConfig;
use devices::dcc::ADC;
use embassy_executor::Spawner;

use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_backtrace as _;
use esp_hal::dma::{Dma, DmaRxBuf, DmaTxBuf};
use esp_hal::gpio::{AnyPin, NoPin, OutputPin};
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::rmt::{Rmt, TxChannelConfig, TxChannelCreatorAsync};
use esp_hal::spi::master::{SpiDma, SpiDmaBus};
use esp_hal::spi::{AnySpi, SpiMode};
use esp_hal::timer::AnyTimer;
use esp_hal::{Async, dma_buffers};
use esp_hal::{
    Blocking,
    analog::adc::{Adc, AdcConfig},
    delay::Delay,
    gpio::{Input, Level, Output},
    interrupt::Priority,
    peripherals::SPI2,
    spi::master::Spi,
    timer::timg::TimerGroup,
};
use esp_hal_embassy::InterruptExecutor;
use fugit::RateExtU32;

use devices::pins;
use ssd1322_rs::Orientation;
use tasks::input::{self, EventBuffer};

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // One ADC powers both modes' current sense, so we define it out here and populate it in each section
    let mut adc_config = AdcConfig::new();

    // Start embassy and get peripherals
    let p = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(p.TIMG0);
    let timg1 = TimerGroup::new(p.TIMG1);
    let timers: [AnyTimer; 2] = [timg0.timer0.into(), timg1.timer0.into()];
    esp_hal_embassy::init(timers);

    /////////////
    //  Setup  //
    /////////////

    // Functions to block out initialization for easier reading
    let init_operations_dcc = || {
        use devices::dcc::operations::*;
        critical_section::with(|cs| {
            // Add current sense to ADC and add the ADC pin as a resource
            CURRENT_SENSE.replace(
                cs,
                Some(
                    adc_config
                        .enable_pin(p.GPIO0, esp_hal::analog::adc::Attenuation::Attenuation11dB),
                ),
            ); // TODO: What attenuation do I need?

            // Add enable pin with a default level set to high to enable transmission
            // TODO: probably want to make this start low, check for issues, then switch on
            ENABLE.replace(cs, Some(Output::new_typed(p.GPIO1, Level::High)));
        });
    };

    let init_rmt = || {
        let rmt = Rmt::new(p.RMT, 80u32.MHz()).unwrap().into_async();
        let tx_pin: <dcc::Operations as devices::pins::dcc::Mode>::Data =
            Output::new_typed(p.GPIO2, Level::Low);
        rmt.channel0
            .configure(tx_pin, esp_hal::rmt::TxChannelConfig {
                clk_divider: 80,
                idle_output_level: true,
                ..TxChannelConfig::default()
            })
            .unwrap()
    };

    let init_display = async || {
        // SPI
        let (sck, mosi): (pins::spi::Sck, pins::spi::Mosi) = (
            Output::new_typed(p.GPIO8, Level::Low),
            Output::new_typed(p.GPIO10, Level::Low),
        );

        let dma = Dma::new(p.DMA);
        let dma_channel = dma.channel0;
        let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
        let spi_bus = Spi::<Async>::new_typed_with_config(
            AnySpi::from(p.SPI2),
            esp_hal::spi::master::Config {
                frequency: 8_000_000u32.Hz(),
                ..Default::default()
            },
        )
        .with_sck(sck)
        .with_mosi(mosi)
        .with_dma(dma_channel.configure(false, esp_hal::dma::DmaPriority::Priority1));

        let dc: pins::spi::Dc = Output::new_typed(p.GPIO7, Level::Low);
        let res: pins::spi::Res = Output::new_typed(p.GPIO9, Level::Low);

        let spi_dma: SpiDmaBus<Async> = SpiDmaBus::new(
            spi_bus,
            DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap(),
            DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap(),
        ); // = SpiDmaBus::new(spi_dma, rx_buf, tx_buf);

        let spi_dev = ExclusiveDevice::new_no_delay(spi_dma, NoPin).unwrap();

        let mut display = ssd1322_rs::SSD1322::new(spi_dev, dc, res, NoPin, Default::default());
        display
            .init_default(&mut embassy_time::Delay)
            .await
            .unwrap();

        display
    };

    /////////////
    //  Start  //
    /////////////

    // DCC pins and devices
    // TODO: Set up service mode pins
    init_operations_dcc();

    // TODO: init_service_dcc

    // Populate the ADC device with our current sense pins now that they have been initialized for both modes
    critical_section::with(|cs| {
        ADC.replace(cs, Some(Adc::new(p.ADC1, adc_config)));
    });

    // Create the RMT remote control device that we'll use to send the DCC signal
    let rmt = init_rmt();

    // Create OLED display via SPI
    let display = init_display().await;

    // Create the rotary encoder device
    let (dt, clk): (pins::rotary_encoder::Data, pins::rotary_encoder::Clock) = (
        Input::new_typed(p.GPIO20, esp_hal::gpio::Pull::None),
        Input::new_typed(p.GPIO6, esp_hal::gpio::Pull::None),
    );
    let rotary_encoder =
        rotary_encoder_embedded::RotaryEncoder::new(dt, clk).into_angular_velocity_mode();

    // Create button

    let sw: pins::rotary_encoder::Switch = Input::new_typed(p.GPIO21, esp_hal::gpio::Pull::Down);
    let button_config: ButtonConfig<embassy_time::Duration> = button_driver::ButtonConfig {
        mode: button_driver::Mode::PullDown,
        ..Default::default()
    };
    let button = button_driver::Button::new(sw, button_config);

    let event_queue = EventBuffer::new();

    critical_section::with(|cs| input::EVENTS.replace(cs, Some(event_queue)));

    /////////////
    //  Tasks  //
    /////////////

    // DCC transmission related tasks
    static EXECUTOR: static_cell::StaticCell<InterruptExecutor<2>> = static_cell::StaticCell::new();
    let sw_ints = SoftwareInterruptControl::new(p.SW_INTERRUPT);
    let executor = InterruptExecutor::new(sw_ints.software_interrupt2);
    let executor = EXECUTOR.init(executor);
    let high_priority = executor.start(Priority::Priority3);

    high_priority.must_spawn(tasks::dcc_operations_transmit(
        &devices::dcc::operations::DRIVER,
    ));
    high_priority.must_spawn(tasks::dcc_operations_step_v3(rmt));

    spawner.must_spawn(tasks::input::process_rotary_input(
        rotary_encoder,
        &input::EVENTS,
        900.Hz(),
    ));
    spawner.must_spawn(tasks::input::process_button_input(
        button,
        &input::EVENTS,
        900.Hz(),
    ));
    // spawner.must_spawn(tasks::input::input_debug_info(&input::EVENTS, 1.Hz()));
    spawner.must_spawn(tasks::display::update_display(
        display,
        &input::EVENTS,
        60.Hz(),
    ));
}
