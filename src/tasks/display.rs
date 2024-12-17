use crate::{
    devices::{Global, types},
    input,
};
use embassy_time::{Duration, Ticker};
use embedded_graphics::{
    mono_font::ascii,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
};
use fugit::HertzU32;
use kolibri_embedded_gui::{
    button::Button,
    label::Label,
    prelude::*,
    style::medsize_rgb565_style,
    ui::{Interaction, Ui},
};

#[embassy_executor::task]
pub async fn update_display(
    mut display: types::Display<'static>,
    events: &'static Global<input::EventBuffer>,
    refresh_rate: fugit::HertzU32,
) {
    let mut ticker = Ticker::every(Duration::from_micros(
        refresh_rate
            .into_duration::<1, { embassy_time::TICK_HZ as u32 }>()
            .to_micros() as u64,
    ));

    // input handling variables
    let mut mouse_down = false;
    let mut last_down = false;
    let mut location = Point::new(0, 0);

    // counter for incrementing thingy
    let mut i = 0u8;

    let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
    ui.clear_background().unwrap();

    loop {
        // create UI (needs to be done each frame)
        let mut ui = Ui::new_fullscreen(&mut display, medsize_rgb565_style());
        // handle input
        match (last_down, mouse_down, location) {
            (false, true, loc) => {
                ui.interact(Interaction::Click(loc));
            }
            (true, true, loc) => {
                ui.interact(Interaction::Drag(loc));
            }
            (true, false, loc) => {
                ui.interact(Interaction::Release(loc));
            }
            (false, false, loc) => {
                ui.interact(Interaction::Hover(loc));
            }
        }

        // clear UI background (for non-incremental redrawing framebuffered applications)
        ui.clear_background().ok();

        last_down = mouse_down;

        // === ACTUAL UI CODE STARTS HERE ===

        ui.add(Label::new("abcdefg").with_font(ascii::FONT_10X20));

        ui.add(Label::new("Basic Counter (7LOC)"));

        if ui.add_horizontal(Button::new("-")).clicked() {
            i = i.saturating_sub(1);
        }

        let mut buffer = itoa::Buffer::new();
        let num = buffer.format(i);
        ui.add_horizontal(Label::new(num));
        if ui.add_horizontal(Button::new("+")).clicked() {
            i = i.saturating_add(1);
        }
        display.flush().unwrap();
        ticker.next().await;

        // // === ACTUAL UI CODE ENDS HERE ===

        // // simulator window update
        // window.update(&display);

        // // take input, and quit application if necessary
        // for evt in window.events() {
        //     match evt {
        //         SimulatorEvent::KeyUp { .. } => {}
        //         SimulatorEvent::KeyDown { .. } => {}
        //         SimulatorEvent::MouseButtonUp { mouse_btn, point } => {
        //             if let MouseButton::Left = mouse_btn {
        //                 mouse_down = false;
        //             }
        //             location = point;
        //         }
        //         SimulatorEvent::MouseButtonDown { mouse_btn, point } => {
        //             if let MouseButton::Left = mouse_btn {
        //                 mouse_down = true;
        //             }
        //             location = point;
        //         }
        //         SimulatorEvent::MouseWheel { .. } => {}
        //         SimulatorEvent::MouseMove { point } => {
        //             location = point;
        //         }
        //         SimulatorEvent::Quit => break 'outer,
        //     }
        // }
    }
    // Ok(())
}
