use core::borrow::Borrow;

use embassy_time::{Duration, Ticker};
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
    let mut app = crate::ui::App {
        ui: crate::ui::Ui {
            target: display,
            model: 0,
            index: 0,
            events,
        },
        refresh_rate,
        view_index: 0,
        views: [View::new(|ui, view| {
            crate::ui::Speed { speed: ui.model }.show(ui, view).unwrap();
        })],
        clear_color: Rgb565::BLACK,
        flush: |target| {
            target.flush().ok();
        },
        events,
    };

    app.run().await;
}
