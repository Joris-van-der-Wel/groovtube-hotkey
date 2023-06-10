use iced::widget::button::Appearance;
use iced_native::widget::button;
use iced::{Color, Theme};

pub struct TextButtonStyleSheet;

impl button::StyleSheet for TextButtonStyleSheet {
    type Style = Theme;

    fn active(&self, _style: &Self::Style) -> Appearance {
        Appearance {
            shadow_offset: Default::default(),
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: Color::BLACK,
        }
    }
}