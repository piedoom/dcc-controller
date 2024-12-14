use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use fugit::HertzU32;

use crate::{
    devices::{Global, types},
    ui::{Component, View},
};

use super::input;

#[embassy_executor::task]
pub async fn update_display(
    display: types::Display<'static>,
    events: &'static Global<input::EventBuffer>,
    refresh_rate: HertzU32,
) {
    // Delay to achieve the desired refresh rate
    let mut app = crate::ui::App::new(
        display,
        events,
        |target| {
            target.flush().ok();
        },
        Rgb565::BLACK,
        refresh_rate,
        [View::new(|ui, view| {
            crate::ui::Speed.show(ui, view).unwrap();
        })],
    );

    app.run().await;
}
