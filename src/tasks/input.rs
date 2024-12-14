use critical_section::Mutex;
use embassy_time::{Duration, Instant, Ticker, Timer};
use esp_println::println;
use fugit::HertzU32;
use heapless::spsc;
use ringbuffer::{ConstGenericRingBuffer, RingBuffer};
use rotary_encoder_embedded::angular_velocity::Velocity;

use crate::devices::{Global, types};

const EVENT_QUEUE_SIZE: usize = 8;

pub type EventBuffer = ConstGenericRingBuffer<InputEvent, EVENT_QUEUE_SIZE>;

pub static EVENTS: Global<EventBuffer> = crate::devices::default();

#[derive(Debug)]
pub enum InputEvent {
    Left(Velocity),
    Right(Velocity),
    Click,
    DoubleClick,
    TripleClick,
}

#[embassy_executor::task]
pub async fn process_input(
    mut button: types::Button<'static>,
    mut rotary_encoder: types::RotaryEncoder<'static>,
    events: &'static Global<EventBuffer>,
    polling_rate: HertzU32,
) {
    loop {
        critical_section::with(|cs| {
            button.tick();
            rotary_encoder.decay_velocity();
            let direction = rotary_encoder.update(Instant::now().as_millis());
            let mut input_events = events.borrow(cs).borrow_mut();
            if let Some(input_events) = input_events.as_mut() {
                match direction {
                    rotary_encoder_embedded::Direction::None => (),
                    rotary_encoder_embedded::Direction::Clockwise => {
                        println!("{}", rotary_encoder.velocity());
                        input_events.push(InputEvent::Left(rotary_encoder.velocity()))
                    }
                    rotary_encoder_embedded::Direction::Anticlockwise => {
                        println!("{}", rotary_encoder.velocity());
                        input_events.push(InputEvent::Right(rotary_encoder.velocity()))
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
