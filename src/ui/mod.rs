// use dcc_rs::packets;

use embassy_time::{Duration, Ticker};
use embedded_graphics::{Drawable, pixelcolor::Rgb565, prelude::*};
use ringbuffer::RingBuffer;

use crate::{
    devices::Global,
    tasks::input::{self, EventBuffer},
};

pub struct App<D: DrawTarget, M, const VIEWS: usize> {
    pub target: D,
    pub model: M,
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
            let _ = self.target.clear(self.clear_color);

            // Render
            // Assume that `view_index` is valid
            let view = &mut self.views[self.view_index];

            // Dequeue all events and apply to the view
            critical_section::with(|cs| {
                for event in self
                    .events
                    .borrow(cs)
                    .borrow_mut()
                    .as_mut()
                    .unwrap()
                    .drain()
                {
                    view.update(event, &mut self.model);
                }
            });

            view.show(&mut self.target, &self.model);

            // Flush
            (self.flush)(&mut self.target);

            // Wait for next frame
            ticker.next().await;
        }
    }
}

pub struct View<D: DrawTarget, M> {
    /// Current index of the cursor for UI items within
    cursor: usize,
    pub ui: fn(&mut D, &M),
    pub update: fn(&mut Self, event: input::InputEvent, model: &mut M),
}

impl<D, M> View<D, M>
where
    D: DrawTarget,
{
    pub fn new(ui: fn(&mut D, &M)) -> Self {
        Self {
            cursor: 0,
            ui,
            update: |_, _, _| (),
        }
    }
    pub fn show(&self, target: &mut D, model: &M) {
        (self.ui)(target, model)
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
    fn show<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>;
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

    fn show<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
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
        .draw(target)?;
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
