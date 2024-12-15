// use dcc_rs::packets;

use embassy_time::{Duration, Ticker};
use embedded_graphics::{
    Drawable, mono_font::MonoTextStyleBuilder, pixelcolor::Rgb565, prelude::*,
};
use ringbuffer::RingBuffer;

use crate::{
    devices::Global,
    tasks::input::{EventBuffer, InputEvent},
};

pub struct Ui<'a, D, M>
where
    D: DrawTarget,
{
    /// Increments for every view rendered
    pub view_incrementer: usize,

    /// Increments every time `show` is called on a UI component, thereby giving all components within a view a unique (unstable) ID. Resets every view
    pub id_incrementer: usize,

    /// The desired view
    pub view_cursor: usize,

    /// The desired component within a view
    pub id_cursor: usize,

    pub events: &'static Global<EventBuffer>,

    pub model: &'a mut M,

    pub target: &'a mut D,
}

impl<'a, D, M> Ui<'a, D, M>
where
    D: DrawTarget,
{
    pub fn new(model: &'a mut M, events: &'static Global<EventBuffer>, target: &'a mut D) -> Self {
        Self {
            view_incrementer: 0,
            id_incrementer: 0,
            view_cursor: 0,
            id_cursor: 0,
            events,
            model,
            target,
        }
    }

    /// Determines whether the current context is currently active
    pub fn active(&self) -> bool {
        self.view_cursor == self.view_incrementer && self.id_cursor == self.id_incrementer
    }
}

pub struct App<D: DrawTarget, M> {
    pub target: D,
    pub refresh_rate: fugit::HertzU32,
    pub view_index: usize,
    pub flush: fn(&mut D),
    pub clear_color: D::Color,
    pub model: M,
    pub show: fn(&mut Ui<D, M>),
    // pub messages:
    pub events: &'static Global<EventBuffer>,
}

impl<D, M> App<D, M>
where
    D: DrawTarget,
{
    pub fn new(
        target: D,
        events: &'static Global<EventBuffer>,
        flush: fn(&mut D),
        clear_color: D::Color,
        model: M,
        refresh_rate: fugit::HertzU32,
    ) -> Self {
        Self {
            refresh_rate,
            view_index: 0,
            flush,
            clear_color,
            target,
            model,
            events,
            show: |_| {},
        }
    }

    pub fn show(mut self, f: fn(&mut Ui<D, M>)) -> Self {
        self.show = f;
        self
    }

    /// Start running the display and update periodically
    pub async fn run(&mut self) {
        let mut ticker = Ticker::every(Duration::from_micros(
            self.refresh_rate
                .into_duration::<1, { embassy_time::TICK_HZ as u32 }>()
                .to_micros() as u64,
        ));

        // Create a UI context
        let mut ui = Ui::new(&mut self.model, self.events, &mut self.target);

        loop {
            // Clear previous frame
            let _ = ui.target.clear(self.clear_color);

            // Render UI
            (self.show)(&mut ui);

            // Flush
            (self.flush)(ui.target);

            // Wait for next frame
            ticker.next().await;
        }
    }
}

pub struct View;

impl View {
    pub fn show<D, M>(&self, ui: &mut Ui<D, M>, f: fn(&mut Ui<D, M>))
    where
        D: DrawTarget,
    {
        // Reset the ID incrementer since it is per-view
        ui.id_incrementer = 0;
        (f)(ui);
        // Increment the view ID
        ui.view_incrementer += 1;
    }
}

pub trait Component {
    type Color = Rgb565;
    type Properties = ();

    fn render<D, M>(&self, ui: &mut Ui<D, M>) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>;
    fn show<D, M>(
        &mut self,
        ui: &mut Ui<D, M>,
        react: fn(&mut M, InputEvent),
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // If active, apply events
        // if ui.active() { TODO: reenable
        // Dequeue all events and apply to the view
        critical_section::with(|cs| {
            for event in ui.events.borrow(cs).borrow_mut().as_mut().unwrap().drain() {
                (react)(ui.model, event);
            }
        });
        // }

        self.render(ui)?;
        // Increment index
        ui.id_incrementer += 1;
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
    fn render<D, M>(&self, ui: &mut Ui<D, M>) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut buffer = itoa::Buffer::new();
        let speed = buffer.format(self.speed);
        embedded_graphics::text::Text::new(
            speed,
            ui.target.bounding_box().center(),
            MonoTextStyleBuilder::new()
                .font(&embedded_graphics::mono_font::ascii::FONT_10X20)
                .text_color(Rgb565::WHITE)
                .build(),
        )
        .draw(ui.target)?;
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
        .draw(ui.target)?;

        Ok(())
    }

    type Color = Rgb565;

    type Properties = ();
}
