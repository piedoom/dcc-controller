#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use core::str;

use button_driver::{Button, ButtonConfig};
use dcc_rs::{
    DccInterruptHandler,
    packets::{Direction, SerializeBuffer, SpeedAndDirection},
};
use devices::dcc::*;
use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use esp_backtrace as _;
use esp_hal::{
    self as hal, Async, Blocking,
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

mod devices;
pub(crate) mod pins;

const LCD_ADDRESS: u8 = 0x3f; // Also might be 0x27

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // peripherals
    let p = esp_hal::init(esp_hal::Config::default());

    // Set up timer for embassy
    let timg0 = TimerGroup::new(p.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // DCC pins and devices

    // One ADC powers both modes' current sense, so we define it out here and populate it in each section
    let mut adc_config = AdcConfig::new();

    // Set up operations mode pins
    {
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
            timer0.set_interrupt_handler(dcc_operations_step);
            interrupt::enable(Interrupt::TG0_T0_LEVEL, Priority::Priority1).unwrap();
            timer0.load_value(500u64.millis()).unwrap();
            timer0.start();
            timer0.listen();

            TIMER.replace(cs, Some(timer0));
        });
    }

    // TODO: Set up service mode pins
    {}

    // Populate the ADC device with our current sense pins now that they have been initialized for both modes
    critical_section::with(|cs| {
        ADC.replace(cs, Some(Adc::new(p.ADC1, adc_config)));
    });

    // Set up general use pins

    // SPI
    let (sck, mosi): (pins::spi::Sck, pins::spi::Mosi) = (
        Output::new_typed(p.GPIO8, Level::Low),
        Output::new_typed(p.GPIO10, Level::Low),
    );
    let spi = Spi::<Blocking, SPI2>::new_typed_with_config(p.SPI2, esp_hal::spi::master::Config {
        frequency: 50_000_000u32.Hz(),
        ..Default::default()
    })
    .with_sck(sck)
    .with_mosi(mosi);

    // Create OLED interface via SPI

    let mut rst: pins::spi::Res = Output::new_typed(p.GPIO9, Level::Low);
    let dc: pins::spi::Dc = Output::new_typed(p.GPIO7, Level::Low);
    let mut display = Ssd1331::new(spi, dc, DisplayRotation::Rotate0);
    let mut delay = Delay::new();
    display.reset(&mut rst, &mut delay).unwrap();
    display.set_rotation(DisplayRotation::Rotate180).unwrap();
    display.init().unwrap();
    display.flush().unwrap();
    let (w, h) = display.dimensions();
    embedded_graphics::primitives::Triangle::new(
        Point::new(8, 16 + 16),
        Point::new(8 + 16, 16 + 16),
        Point::new(8 + 8, 16),
    )
    .into_styled(embedded_graphics::primitives::PrimitiveStyle::with_stroke(
        Rgb565::RED,
        2,
    ))
    .draw(&mut display)
    .unwrap();
    display.flush().unwrap();

    // Create the rotary encoder device

    let (dt, clk): (pins::rotary_encoder::Data, pins::rotary_encoder::Clock) = (
        Input::new_typed(p.GPIO20, esp_hal::gpio::Pull::Up),
        Input::new_typed(p.GPIO6, esp_hal::gpio::Pull::Up),
    );
    let rotary_encoder =
        rotary_encoder_embedded::RotaryEncoder::new(dt, clk).into_angular_velocity_mode();
    critical_section::with(|cs| ROTARY_ENCODER.replace(cs, Some(rotary_encoder)));

    // Create button
    let sw: pins::rotary_encoder::Switch = Input::new_typed(p.GPIO21, esp_hal::gpio::Pull::Down);
    let button_config: ButtonConfig<embassy_time::Duration> = button_driver::ButtonConfig {
        mode: button_driver::Mode::PullDown,
        ..Default::default()
    };
    let button: devices::types::Button = Button::<_, Instant, Duration>::new(sw, button_config);
    critical_section::with(|cs| BUTTON.replace(cs, Some(button)));

    loop {
        //info!("tx, addr = {}", addr);
        // pop a new chunk of data into the buffer
        let pkt = SpeedAndDirection::builder()
            .address(10)
            .unwrap()
            .speed(14)
            .unwrap()
            .direction(Direction::Forward)
            .build();
        let mut buffer = SerializeBuffer::default();
        let len = pkt.serialize(&mut buffer).unwrap();
        // println!("{:?}", buffer);

        critical_section::with(|cs| {
            operations::DRIVER
                .borrow_ref_mut(cs)
                .as_mut()
                .unwrap()
                .write(buffer.get(0..len).unwrap())
                .unwrap();
        });

        Timer::after_millis(15).await; // Retransmit after minimum amount of time in spec
    }
}

// Transmit DCC signal
#[handler]
fn dcc_operations_step() {
    let tx_buffer = critical_section::with(|cs| operations::TX_BUFFER.take(cs));
    let mut dcc_handler = critical_section::with(|cs| operations::DRIVER.take(cs).unwrap());

    // Only write to the handler if we have new data in the buffer
    if let Some((new_data, len)) = tx_buffer {
        dcc_handler.write(&new_data[..len]).unwrap();
    }

    // Set the delay until the next level change
    let new_delay = dcc_handler.tick().unwrap();

    let timer = critical_section::with(|cs| operations::TIMER.take(cs).unwrap());

    timer.load_value((new_delay as u64).micros()).unwrap();
    timer.start();

    timer.clear_interrupt();

    // Replace devices
    critical_section::with(|cs| operations::DRIVER.replace(cs, Some(dcc_handler)));
    critical_section::with(|cs| operations::TIMER.replace(cs, Some(timer)));
}

#[embassy_executor::task]
async fn update_display() {
    // // Take SPI device and create LCD
    // let mut delay = Delay::new();
    // let mut i2c = critical_section::with(|cs| devices::I2C.take(cs).unwrap());
    // let mut lcd = lcd_lcm1602_i2c::LCD16x4::new(&mut i2c, &mut delay)
    //     .with_address(LCD_ADDRESS)
    //     .with_cursor_on(false) // no visible cursor
    //     .init()
    //     .unwrap();

    // let spi_device = SpiDevice::new(bus, cs);

    // loop {
    //     lcd.set_cursor(0, 0).unwrap();
    //     screen_buffer[3] = 'w' as u8;
    //     lcd.write_str("a");
    //     // lcd.write_str(str::from_utf8(&screen_buffer).unwrap())
    //     //     .unwrap();

    //     // critical_section::with(|cs| {
    //     //     // Replace screen buffer
    //     //     globals::SCREEN_BUFFER.replace(cs, Some(screen_buffer));
    //     //     // Replace I2C device
    //     //     globals::I2C.replace(cs, Some(i2c));
    //     // });

    //     Timer::after_millis(30).await;
    // }
}
