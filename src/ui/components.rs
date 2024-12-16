use crate::ui::{Component, Ui};
use arrayvec::ArrayVec;
use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{self, Rectangle},
};

pub struct Decoder {
    address: u8,
    active: bool,
}

pub struct AddressSelector<'a> {
    pub decoders: &'a ArrayVec<Decoder, 127>,
    pub rect: Option<Rectangle>,
}

impl<'a> AddressSelector<'a> {
    pub fn new(decoders: &'a ArrayVec<Decoder, 127>) -> Self {
        Self {
            decoders,
            rect: None,
        }
    }
    pub fn with_rect(mut self, rect: Rectangle) -> Self {
        self.rect = Some(rect);
        self
    }
}

impl<'a> Component for AddressSelector<'a> {
    type Color = Rgb565;

    fn render<D, M>(&self, ui: &mut Ui<D, M>) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        todo!()
    }
}

pub struct Speed {
    pub speed: u8,
    pub rect: Rectangle,
}

impl Component for Speed {
    type Color = Rgb565;

    fn render<D, M>(&self, ui: &mut Ui<D, M>) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let text_color = if ui.active() {
            Rgb565::RED
        } else {
            Rgb565::CSS_GRAY
        };

        let mut buffer = itoa::Buffer::new();
        let speed = buffer.format(self.speed);

        // Display an outline if hovered or active
        let outline = if ui.active() {
            Some(Rgb565::CSS_BROWN)
        } else if ui.hovered() {
            Some(Rgb565::CSS_DIM_GRAY)
        } else {
            None
        };
        if let Some(outline) = outline {
            self.rect
                .into_styled(primitives::PrimitiveStyle::with_stroke(outline, 4))
                .draw(ui.target)?;
        }

        // Show text speed
        embedded_graphics::text::Text::new(
            speed,
            self.rect.center(),
            MonoTextStyleBuilder::new()
                .font(&embedded_graphics::mono_font::ascii::FONT_10X20)
                .text_color(text_color)
                .build(),
        )
        .draw(ui.target)?;
        // Generate graphics
        primitives::Triangle::new(
            Point::new(8 + self.speed as i32, 16 + 16),
            Point::new(8 + self.speed as i32 + 16, 16 + 16),
            Point::new(8 + self.speed as i32 + 8, 16),
        )
        .into_styled(primitives::PrimitiveStyle::with_stroke(
            Self::Color::YELLOW,
            2,
        ))
        .draw(ui.target)?;

        Ok(())
    }
}
