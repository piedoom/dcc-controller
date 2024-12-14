// use dcc_rs::packets;

use core::str;

use embassy_time::{Duration, Ticker};
use embedded_graphics::{
    Drawable,
    mono_font::{MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::Rgb565,
    prelude::*,
};
use libm::powf;
use ringbuffer::RingBuffer;
use rotary_encoder_embedded::angular_velocity::Velocity;

use crate::{
    devices::Global,
    tasks::input::{self, EventBuffer},
};

pub struct Ui<D: DrawTarget, M> {
    pub target: D,
    /// Current index of component within a view
    pub index: usize,
    pub events: &'static Global<EventBuffer>,
    pub model: M,
}

impl<D, M> Ui<D, M>
where
    D: DrawTarget,
    M: Default,
{
    pub fn new(target: D, events: &'static Global<EventBuffer>) -> Self {
        Self {
            target,
            index: 0,
            events,
            model: M::default(),
        }
    }
}

pub struct App<D: DrawTarget, M, const VIEWS: usize> {
    pub ui: Ui<D, M>,
    pub refresh_rate: fugit::HertzU32,
    pub view_index: usize,
    pub views: [View<D, M>; VIEWS],
    pub flush: fn(&mut D),
    pub clear_color: D::Color,
    // pub events: &'static Global<EventBuffer>,
}

impl<D, M, const VIEWS: usize> App<D, M, VIEWS>
where
    D: DrawTarget,
    M: Default,
{
    pub fn new(
        target: D,
        events: &'static Global<EventBuffer>,
        flush: fn(&mut D),
        clear_color: D::Color,
        refresh_rate: fugit::HertzU32,
        views: [View<D, M>; VIEWS],
    ) -> Self {
        Self {
            ui: Ui::new(target, events),
            refresh_rate,
            view_index: 0,
            views,
            flush,
            clear_color,
        }
    }

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
}

impl<D, M> View<D, M>
where
    D: DrawTarget,
{
    pub fn new(ui: fn(&mut Ui<D, M>, &Self)) -> Self {
        Self { cursor: 0, ui }
    }
    pub fn show(&self, ui: &mut Ui<D, M>) {
        (self.ui)(ui, self)
    }
}

pub trait Component {
    type Color = Rgb565;
    type Properties = ();
    type Model = ();
    fn on_left<D>(&mut self, ui: &mut Ui<D, Self::Model>, velocity: Velocity)
    where
        D: DrawTarget<Color = Self::Color>;
    fn on_right<D>(&mut self, ui: &mut Ui<D, Self::Model>, velocity: Velocity)
    where
        D: DrawTarget<Color = Self::Color>;
    fn render<D>(&self, ui: &mut Ui<D, Self::Model>, active: bool) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>;
    fn show<D>(
        &mut self,
        ui: &mut Ui<D, Self::Model>,
        view: &View<D, Self::Model>,
    ) -> Result<(), D::Error>
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
                    input::InputEvent::Left(velocity) => self.on_left(ui, velocity),
                    input::InputEvent::Right(velocity) => self.on_right(ui, velocity),
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

pub struct Speed;

impl Component for Speed {
    type Model = usize;
    fn render<D>(&self, ui: &mut Ui<D, Self::Model>, active: bool) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut buffer = itoa::Buffer::new();
        let speed = buffer.format(ui.model);
        embedded_graphics::text::Text::new(
            speed,
            ui.target.bounding_box().center(),
            MonoTextStyleBuilder::new()
                .font(&embedded_graphics::mono_font::ascii::FONT_10X20)
                .text_color(Rgb565::WHITE)
                .build(),
        )
        .draw(&mut ui.target)?;
        // Generate graphics
        embedded_graphics::primitives::Triangle::new(
            Point::new(8 + ui.model as i32, 16 + 16),
            Point::new(8 + ui.model as i32 + 16, 16 + 16),
            Point::new(8 + ui.model as i32 + 8, 16),
        )
        .into_styled(embedded_graphics::primitives::PrimitiveStyle::with_stroke(
            Self::Color::YELLOW,
            2,
        ))
        .draw(&mut ui.target)?;

        Ok(())
    }

    type Color = Rgb565;

    fn on_left<D>(&mut self, ui: &mut Ui<D, Self::Model>, velocity: Velocity)
    where
        D: DrawTarget<Color = Self::Color>,
    {
        ui.model = ui
            .model
            .saturating_sub(powf(1.2f32, velocity * 10f32) as usize);
    }

    fn on_right<D>(&mut self, ui: &mut Ui<D, Self::Model>, velocity: Velocity)
    where
        D: DrawTarget<Color = Self::Color>,
    {
        ui.model = ui
            .model
            .saturating_add(powf(1.2f32, velocity * 10f32) as usize);
    }
}
