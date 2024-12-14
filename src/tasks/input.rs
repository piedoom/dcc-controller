use core::{borrow::Borrow, cell::RefCell};

use critical_section::Mutex;
use embassy_time::{Duration, Instant, Ticker, Timer};
use esp_println::println;
use fugit::HertzU32;
use heapless::spsc;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};

use crate::{
    BUTTON,
    devices::{Global, pins, types},
};

const EVENT_QUEUE_SIZE: usize = 8;

pub type EventBuffer = ConstGenericRingBuffer<InputEvent, EVENT_QUEUE_SIZE>;

pub static EVENTS: Global<EventBuffer> = crate::devices::default();

#[derive(Debug)]
pub enum InputEvent {
    Left,
    Right,
    Click,
    DoubleClick,
    TripleClick,
}

#[embassy_executor::task]
pub async fn process_input(
    button: &'static Global<types::Button<'static>>,
    rotary_encoder: &'static Global<types::RotaryEncoder<'static>>,
    events: &'static Global<EventBuffer>,
    polling_rate: HertzU32,
) {
    loop {
        critical_section::with(|cs| {
            let mut rotary_encoder = rotary_encoder.borrow(cs).borrow_mut();
            let mut button = button.borrow(cs).borrow_mut();
            let mut input_events = events.borrow(cs).borrow_mut();
            if let (Some(button), Some(input_events), Some(rotary_encoder)) = (
                button.as_mut(),
                input_events.as_mut(),
                rotary_encoder.as_mut(),
            ) {
                button.tick();
                let direction = rotary_encoder.update(Instant::now().as_millis());
                match direction {
                    rotary_encoder_embedded::Direction::None => (),
                    rotary_encoder_embedded::Direction::Clockwise => {
                        input_events.push(InputEvent::Left)
                    }
                    rotary_encoder_embedded::Direction::Anticlockwise => {
                        input_events.push(InputEvent::Right)
                    }
                }
                if button.is_clicked() {
                    println!("a");
                    input_events.push(InputEvent::Click);
                } else if button.is_double_clicked() {
                    input_events.push(InputEvent::DoubleClick);
                } else if button.is_triple_clicked() {
                    input_events.push(InputEvent::TripleClick);
                }
            }
        });

        // Delay to achieve the desired polling rate
        Timer::after_micros(
            polling_rate
                .into_duration::<1, { embassy_time::TICK_HZ as u32 }>()
                .to_micros() as u64,
        )
        .await;
    }
}

#[embassy_executor::task]
pub async fn input_debug_info(events: &'static Global<EventBuffer>, refresh_rate: HertzU32) {
    // Delay to achieve the desired refresh rate
    let mut ticker = Ticker::every(Duration::from_micros(
        refresh_rate
            .into_duration::<1, { embassy_time::TICK_HZ as u32 }>()
            .to_micros() as u64,
    ));
    loop {
        critical_section::with(|cs| {
            let events = events.borrow(cs).borrow();
            events.as_ref().unwrap().iter().for_each(|event| {
                println!("{:?}", event);
            });
        });

        ticker.next().await;
    }
}
