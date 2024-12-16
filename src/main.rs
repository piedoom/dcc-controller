#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(associated_type_defaults)]

mod dcc;
mod devices;
pub mod tasks;
pub mod ui;

use button_driver::ButtonConfig;
use dcc_rs::DccInterruptHandler;
use devices::dcc::*;
use embassy_executor::Spawner;

use esp_backtrace as _;
use esp_hal::{
    self as hal, Blocking,
    analog::adc::{Adc, AdcConfig},
    delay::Delay,
    gpio::{Input, Level, Output},
    interrupt::{self, Priority},
    peripherals::{Interrupt, SPI2},
    spi::master::Spi,
    timer::timg::TimerGroup,
};
use fugit::RateExtU32;
use hal::prelude::*;

use ssd1331::{DisplayRotation, Ssd1331};

use devices::pins;
use tasks::input::{self, EventBuffer};

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // peripherals
    let p = esp_hal::init(esp_hal::Config::default());

    // Set up timer for embassy
    let timg0 = TimerGroup::new(p.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // One ADC powers both modes' current sense, so we define it out here and populate it in each section
    let mut adc_config = AdcConfig::new();

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

            // Create DCC interrupt driver and timer
            DRIVER.replace(
                cs,
                Some(DccInterruptHandler::new(Output::new_typed(
                    p.GPIO2,
                    Level::Low,
                ))),
            );

            let timg1 = TimerGroup::new(p.TIMG1);
            let timer0 = timg1.timer0;

            // Bitbang interrupt handler for DCC transmission
            timer0.set_interrupt_handler(tasks::dcc_operations_step);
            interrupt::enable(Interrupt::TG0_T0_LEVEL, Priority::Priority1).unwrap();
            timer0.load_value(500u64.millis()).unwrap();
            timer0.start();
            timer0.listen();

            TIMER.replace(cs, Some(timer0));
        });
    };

    let init_display = || {
        // SPI
        let (sck, mosi): (pins::spi::Sck, pins::spi::Mosi) = (
            Output::new_typed(p.GPIO8, Level::Low),
            Output::new_typed(p.GPIO10, Level::Low),
        );

        let spi =
            Spi::<Blocking, SPI2>::new_typed_with_config(p.SPI2, esp_hal::spi::master::Config {
                frequency: 50_000_000u32.Hz(),
                ..Default::default()
            })
            .with_sck(sck)
            .with_mosi(mosi);

        let dc: pins::spi::Dc = Output::new_typed(p.GPIO7, Level::Low);

        Ssd1331::new(spi, dc, DisplayRotation::Rotate0)
    };

    // Begin calling setup functions and create devices

    // DCC pins and devices
    // TODO: Set up service mode pins
    init_operations_dcc();

    // Populate the ADC device with our current sense pins now that they have been initialized for both modes
    critical_section::with(|cs| {
        ADC.replace(cs, Some(Adc::new(p.ADC1, adc_config)));
    });

    // Create OLED display via SPI
    // We don't put this in global storage since we'll give exclusive access to the rendering task
    let mut display = init_display();

    // Reset the display and set configuration
    let mut rst: pins::spi::Res = Output::new_typed(p.GPIO9, Level::Low);
    display.flush().unwrap();
    display.reset(&mut rst, &mut Delay::new()).unwrap();
    display.set_rotation(DisplayRotation::Rotate180).unwrap();
    display.init().unwrap();
    display.flush().unwrap();

    // Create the rotary encoder device

    let (dt, clk): (pins::rotary_encoder::Data, pins::rotary_encoder::Clock) = (
        Input::new_typed(p.GPIO20, esp_hal::gpio::Pull::Up),
        Input::new_typed(p.GPIO6, esp_hal::gpio::Pull::Up),
    );
    let rotary_encoder =
        rotary_encoder_embedded::RotaryEncoder::new(dt, clk).into_angular_velocity_mode();

    // Create button

    let sw: pins::rotary_encoder::Switch = Input::new_typed(p.GPIO21, esp_hal::gpio::Pull::Down);
    let button_config: ButtonConfig<embassy_time::Duration> = button_driver::ButtonConfig {
        mode: button_driver::Mode::PullUp,
        ..Default::default()
    };

    let button = button_driver::Button::new(sw, button_config);

    let event_queue = EventBuffer::new();

    critical_section::with(|cs| input::EVENTS.replace(cs, Some(event_queue)));

    // Spawn tasks

    // DCC transmission related tasks
    spawner.must_spawn(tasks::dcc_operations_transmit());

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
