// use dcc_rs::packets;

use core::str;

use embassy_time::{Duration, Ticker};
use embedded_graphics::{
    Drawable,
    mono_font::{MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::Rgb565,
    prelude::*,
};
use ringbuffer::RingBuffer;

use crate::{
    devices::Global,
    tasks::input::{self, EventBuffer},
};

pub struct Ui<D: DrawTarget, M> {
    pub target: D,
    pub model: M,
    /// Current index of component within a view
    pub index: usize,
    pub events: &'static Global<EventBuffer>,
}

pub struct App<D: DrawTarget, M, const VIEWS: usize> {
    pub ui: Ui<D, M>,
    pub refresh_rate: fugit::HertzU32,
    pub view_index: usize,
    pub views: [View<D, M>; VIEWS],
    pub flush: fn(&mut D),
    pub clear_color: D::Color,
    pub events: &'static Global<EventBuffer>,
}

impl<D, M, const VIEWS: usize> App<D, M, VIEWS>
where
    D: DrawTarget,
{
    /// Start running the display and update periodically
    pub async fn run(&mut self) {
        let mut ticker = Ticker::every(Duration::from_micros(
            self.refresh_rate
                .into_duration::<1, { embassy_time::TICK_HZ as u32 }>()
                .to_micros() as u64,
        ));

        loop {
            // Clear previous frame
            let _ = self.ui.target.clear(self.clear_color);

            // Render
            // Assume that `view_index` is valid
            let view = &mut self.views[self.view_index];

            // Reset the view component index counter
            view.cursor = 0;

            view.show(&mut self.ui);

            // Flush
            (self.flush)(&mut self.ui.target);

            // Wait for next frame
            ticker.next().await;
        }
    }
}

pub struct View<D: DrawTarget, M> {
    /// Current index of the cursor for UI items within
    cursor: usize,
    pub ui: fn(&mut Ui<D, M>, &Self),
    pub update: fn(&mut Self, event: input::InputEvent, model: &mut M),
}

impl<D, M> View<D, M>
where
    D: DrawTarget,
{
    pub fn new(ui: fn(&mut Ui<D, M>, &Self)) -> Self {
        Self {
            cursor: 0,
            ui,
            update: |_, _, _| (),
        }
    }
    pub fn show(&self, ui: &mut Ui<D, M>) {
        (self.ui)(ui, self)
    }
    pub fn update(&mut self, event: input::InputEvent, model: &mut M) {
        (self.update)(self, event, model);
    }
}

pub trait Component {
    type Color = Rgb565;
    type Properties = ();
    fn on_left(&mut self);
    fn on_right(&mut self);
    fn render<D, M>(&self, ui: &mut Ui<D, M>, active: bool) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>;
    fn show<D, M>(&mut self, ui: &mut Ui<D, M>, view: &View<D, M>) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // See if active
        let active = ui.index == view.cursor;

        // If active, apply events
        // if active {
        // Dequeue all events and apply to the view
        critical_section::with(|cs| {
            for event in ui.events.borrow(cs).borrow_mut().as_mut().unwrap().drain() {
                match event {
                    input::InputEvent::Left => self.on_left(),
                    input::InputEvent::Right => self.on_right(),
                    // input::InputEvent::Click => self.on_click(),
                    _ => (),
                }
            }
        });
        // }

        self.render(ui, active)?;
        // Increment index
        ui.index += 1;
        Ok(())
    }
    // fn on_right(model: &mut M) {}
    // fn on_focus(model: &mut M) {}
    // fn on_defocus(model: &mut M) {}
    // fn on_click(model: &mut M) {}
}

pub struct Speed {
    pub speed: usize,
}

impl Component for Speed {
    fn on_left(&mut self) {
        self.speed = self.speed.saturating_sub(1);
    }
    fn on_right(&mut self) {
        self.speed = self.speed.saturating_add(1);
    }

    fn render<D, M>(&self, ui: &mut Ui<D, M>, active: bool) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        embedded_graphics::text::Text::new(
            str::from_utf8(&[self.speed as u8]).unwrap(),
            ui.target.bounding_box().center(),
            MonoTextStyleBuilder::new()
                .font(&embedded_graphics::mono_font::ascii::FONT_6X9)
                .text_color(Rgb565::WHITE)
                .build(),
        )
        .draw(&mut ui.target)?;
        // Generate graphics
        embedded_graphics::primitives::Triangle::new(
            Point::new(8 + self.speed as i32, 16 + 16),
            Point::new(8 + self.speed as i32 + 16, 16 + 16),
            Point::new(8 + self.speed as i32 + 8, 16),
        )
        .into_styled(embedded_graphics::primitives::PrimitiveStyle::with_stroke(
            Self::Color::YELLOW,
            2,
        ))
        .draw(&mut ui.target)?;

        Ok(())
    }
}

// pub enum Page {
//     AdjustSpeed(AdjustSpeed),
// }

// pub struct AdjustSpeed {}

// impl Component for AdjustSpeed {
//     type Message = Option<packets::Direction>;

//     fn on_left() -> Self::Message {
//         todo!()
//     }

//     fn on_right() -> Self::Message {
//         todo!()
//     }

//     fn on_press() -> Self::Message {
//         todo!()
//     }
// }
