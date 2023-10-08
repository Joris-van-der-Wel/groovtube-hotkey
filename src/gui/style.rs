use iced::{Color, Theme};
use iced::widget::button::{StyleSheet, Appearance};

pub struct TextButtonStyleSheet;

impl StyleSheet for TextButtonStyleSheet {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            shadow_offset: Default::default(),
            background: None,
            border_radius: 0.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: Color::BLACK,
        }
    }
}
