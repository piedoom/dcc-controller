use crate::{
    devices::{self, Global, dcc::operations, types},
    input,
};
use dcc_rs::packets::{MAX_SPEED, SpeedAndDirection};
use embassy_time::{Duration, Ticker};
use embedded_graphics::{
    geometry::AnchorX,
    mono_font::{self, ascii},
    pixelcolor::{Gray4, Rgb565},
    prelude::*,
    primitives::{Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
};
use fugit::HertzU32;
use kolibri_embedded_gui::{
    button::Button,
    label::Label,
    prelude::*,
    style::{Spacing, Style, medsize_rgb565_style},
    ui::{Interaction, Response, Ui},
};
use ringbuffer::RingBuffer;
use ssd1322_rs::Frame;

pub struct Page {
    cursor: usize,
    cursor_active: bool,
    pub positions: heapless::Vec<Rectangle, 16>,
}

pub trait WithPage {
    fn interact(self, page: &mut Page) -> Self;
}

impl WithPage for Response {
    fn interact(self, page: &mut Page) -> Self {
        page.positions
            .push(self.internal.area)
            .expect("buffer too small");
        self
    }
}

impl Page {
    pub fn clear(&mut self) {
        self.positions.clear();
    }
}

pub struct View<T, DRAW, COL>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
{
    route: T,
    content: fn(&mut Ui<DRAW, COL>),
}

impl<T, DRAW, COL> View<T, DRAW, COL>
where
    DRAW: DrawTarget<Color = COL>,
    COL: PixelColor,
    T: PartialEq,
{
    pub fn new(route: T) -> Self {
        Self {
            route,
            content: |_| (),
        }
    }

    pub fn with(
        &self,
        current_route: &T,
        mut ui: Ui<DRAW, COL>,
        mut f: impl FnMut(&mut Ui<DRAW, COL>),
    ) {
        if *current_route == self.route {
            (f)(&mut ui);
        }
    }
}

#[derive(PartialEq)]
pub enum Route {
    Speed,
}

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

    use ssd1322_rs::{Frame, Orientation, SSD1322, calculate_buffer_size};
    use static_cell::StaticCell;

    const SCREEN_WIDTH: usize = 256;
    const SCREEN_HEIGHT: usize = 64;
    const SCREEN_RECT: Rectangle = Rectangle {
        top_left: Point::zero(),
        size: Size::new(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32),
    };
    const BUF_SIZE: usize = calculate_buffer_size(SCREEN_WIDTH, SCREEN_HEIGHT);

    static FRAME_A: StaticCell<Frame<BUF_SIZE>> = StaticCell::new();

    let frame = FRAME_A.init(Default::default());

    let style = Style {
        background_color: Gray4::BLACK,
        item_background_color: Gray4::BLACK,
        highlight_item_background_color: Gray4::new(1),
        border_color: Gray4::new(100),
        highlight_border_color: Gray4::new(255),
        primary_color: Gray4::WHITE,
        secondary_color: Gray4::WHITE,
        icon_color: Gray4::WHITE,
        text_color: Gray4::WHITE,
        default_widget_height: 12,
        border_width: 1,
        highlight_border_width: 1,
        default_font: mono_font::ascii::FONT_9X15,
        spacing: Spacing {
            item_spacing: Size::new(8, 8),
            button_padding: Size::new(4, 0),
            default_padding: Size::new(8, 8),
            window_border_padding: Size::new(8, 8),
        },
    };
    let mut ui = Ui::new_fullscreen(frame, style);
    let mut cursor = Point::new(10, 10);
    // counter for incrementing thingy

    // Clear events before the first loop
    critical_section::with(|cs| {
        events.borrow_ref_mut(cs).as_mut().unwrap().clear();
    });

    let mut page = Page {
        cursor: 0,
        cursor_active: false,
        positions: Default::default(),
    };

    let mut current_route = Route::Speed;

    loop {
        // create UI (needs to be done each frame)
        let mut ui = Ui::new_fullscreen(frame, style);
        ui.clear_background().unwrap();
        // consume and react to events
        let mut current_events = critical_section::with(|cs| events.take(cs).unwrap());
        critical_section::with(|cs| {
            let mut model = operations::STATE.borrow_ref_mut(cs);
            let model = model.as_mut().unwrap();

            // Always hovers
            if page.positions.get(page.cursor).is_some() {
                ui.interact(Interaction::Hover(page.positions[page.cursor].center()));
            }

            // Set the cursors

            for event in current_events.iter() {
                match event {
                    input::InputEvent::MoveCursor(direction) => {
                        if page.cursor_active {
                            // Widget active
                        } else {
                            // Move page cursor
                            page.cursor = match direction {
                                input::MoveDirection::Left => page.cursor.saturating_sub(1),
                                input::MoveDirection::Right => page.cursor + 1,
                            };
                        }
                    }
                    // input::InputEvent::Right(_) => {
                    //     if page.cursor_active {
                    //         // Widget active
                    //     } else {
                    //         // Move page cursor
                    //         if page.cursor < page.positions.len().saturating_sub(1) {
                    //             page.cursor += 1;
                    //         }
                    //     }
                    // }
                    // input::InputEvent::Click => {
                    //     if page.cursor_active {
                    //         // Widget active
                    //     } else {
                    //         ui.interact(Interaction::Click(page.positions[page.cursor].center()));
                    //     }
                    // }
                    // input::InputEvent::DoubleClick => todo!(),
                    input::InputEvent::Hold => {
                        // ui.interact(Interaction::Click(page.cursor));
                    }
                    _ => (),
                }
            }

            current_events.clear();
            page.clear();

            // // clear UI background (for non-incremental redrawing framebuffered applications)
            // ui.clear_background().ok();

            // === ACTUAL UI CODE STARTS HERE ===

            // ui.add(Label::new("Basic Counter (7LOC)"));

            View::new(Route::Speed).with(&current_route, ui, |ui| {
                ui.unchecked_sub_ui(
                    SCREEN_RECT.resized_width((SCREEN_WIDTH / 2) as u32, AnchorX::Left),
                    |ui| {
                        ui.add_horizontal(Label::new("Address"));
                        ui.new_row();

                        if ui
                            .add_horizontal(Button::new("-"))
                            .interact(&mut page)
                            .down()
                        {
                            model.address = model.address.saturating_sub(1);
                        }

                        let mut buffer = itoa::Buffer::new();
                        let address = buffer.format(model.address);
                        ui.add_horizontal(Label::new(address));

                        if ui
                            .add_horizontal(Button::new("+"))
                            .interact(&mut page)
                            .down()
                        {
                            model.address += 1;
                        }

                        model.address = model.address.min(127);
                        Ok(())
                    },
                )
                .unwrap();

                ui.unchecked_sub_ui(
                    SCREEN_RECT.resized_width((SCREEN_WIDTH / 2) as u32, AnchorX::Right),
                    |ui| {
                        ui.add_horizontal(Label::new("Speed"));
                        ui.new_row();

                        if ui
                            .add_horizontal(Button::new("-"))
                            .interact(&mut page)
                            .down()
                        {
                            model.speed -= 1;
                        }

                        let mut buffer = itoa::Buffer::new();
                        let num = buffer.format(model.speed);
                        ui.add_horizontal(Label::new(num));

                        if ui
                            .add_horizontal(Button::new("+"))
                            .interact(&mut page)
                            .down()
                        {
                            model.speed += 1;
                        }

                        model.speed = model.speed.clamp(-(MAX_SPEED as i8), MAX_SPEED as i8);
                        Ok(())
                    },
                )
                .unwrap();
            });

            critical_section::with(|cs| {
                events.replace(cs, Some(current_events));
                // operations::STATE.replace(cs, Some(model));
            });
        });
        display.flush_frame(frame).await.unwrap();
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
