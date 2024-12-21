#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(associated_type_defaults)]
#![feature(async_closure)]

use decoder::{pins, tasks};
use embassy_executor::Spawner;
use esp_hal::{
    gpio::{Input, Pull},
    rmt::{Rmt, RxChannelConfig, RxChannelCreatorAsync},
    timer::{AnyTimer, timg::TimerGroup},
};
use fugit::RateExtU32;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let p = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(p.TIMG0);
    let timers: AnyTimer = timg0.timer0.into();
    esp_hal_embassy::init(timers);

    let init_rmt = || {
        let rmt = Rmt::new(p.RMT, 80u32.MHz()).unwrap().into_async();
        let tx_pin: pins::Data = Input::new_typed(p.GPIO5, Pull::None);
        rmt.channel2
            .configure(tx_pin, esp_hal::rmt::RxChannelConfig {
                clk_divider: 80,
                ..RxChannelConfig::default()
            })
            .unwrap()
    };

    let rmt = init_rmt();

    spawner.must_spawn(tasks::decode::receive_data(rmt));
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
