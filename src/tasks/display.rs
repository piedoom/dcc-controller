use arrayvec::ArrayVec;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use fugit::HertzU32;

use crate::{
    devices::{Global, types},
    ui::{
        Component, View,
        components::{self, Decoder},
    },
};

use super::input::{self, InputEvent};

#[derive(Default)]
pub struct Model {
    decoders: ArrayVec<Decoder, 127>,
    speed: u8,
}

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
        Model::default(),
        refresh_rate,
    );

    app.show(|ui| {
        View.show(ui, |ui| {
            components::Speed {
                speed: ui.model.speed,
                rect: ui.target.bounding_box(),
            }
            .show(ui, |model, event| match event {
                InputEvent::Left(_velocity) => model.speed -= 1,
                InputEvent::Right(_velocity) => model.speed += 1,
                InputEvent::Hold => model.speed = 0,
                _ => (),
            })
            .unwrap();

            // components::AddressSelector::new(&ui.model.decoders) // TODO: make static?
            //     .show(ui, |model, event| {})
            //     .unwrap();
        });
    })
    .run()
    .await;
}
