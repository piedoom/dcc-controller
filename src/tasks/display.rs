use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use fugit::HertzU32;

use crate::{
    devices::{Global, types},
    ui::{Component, View},
};

use super::input::{self, InputEvent};

#[embassy_executor::task]
pub async fn update_display(
    display: types::Display<'static>,
    events: &'static Global<input::EventBuffer>,
    refresh_rate: HertzU32,
) {
    // Delay to achieve the desired refresh rate
    let app = crate::ui::App::new(
        display,
        events,
        |target| {
            target.flush().ok();
        },
        Rgb565::BLACK,
        0usize,
        refresh_rate,
    );

    app.show(|ui| {
        View.show(ui, |ui| {
            crate::ui::Speed { speed: *ui.model }
                .show(ui, |model, event| match event {
                    InputEvent::Left(_velocity) => *model -= 1,
                    InputEvent::Right(_velocity) => *model += 1,
                    InputEvent::Hold => *model = 0,
                    _ => (),
                })
                .unwrap();
        });
    })
    .run()
    .await;
}
