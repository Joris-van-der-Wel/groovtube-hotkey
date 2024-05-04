use iced::{Border, Color, Shadow, Theme};
use iced::widget::button::{StyleSheet, Appearance};

pub struct TextButtonStyleSheet;

impl StyleSheet for TextButtonStyleSheet {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            shadow_offset: Default::default(),
            background: None,
            text_color: Color::BLACK,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: Shadow::default(),
        }
    }
}
