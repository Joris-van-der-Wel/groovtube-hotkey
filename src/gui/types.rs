use iced::{Event};
use iced::font::{Error as FontError};

use crate::config::types::{BreathDirection, Config};
use crate::device::types::{DeviceEvent};
use crate::sim::types::Button;

#[derive(Debug, Clone)]
pub enum HotkeyModifier {
    Shift,
    Ctrl,
    Meta,
    Alt,
}

#[derive(Debug, Clone)]
pub enum HotkeyChange {
    BreathDirectionChange(BreathDirection),
    ButtonChange(Button),
    ThresholdChange(String),
    ModifierToggle(HotkeyModifier),
    Delete,
}

#[derive(Debug, Clone)]
pub enum Message {
    EventOccurred(Event),
    ApplyDirtyConfig,
    WriteComplete(()),
    SymbolsFontLoadComplete(Result<(), FontError>),
    ConfigLoadComplete(Config),
    ConfigSaveComplete(bool), // true if success, false if failed
    DeviceEvent(DeviceEvent),
    AddHotkey,
    HotkeyChange(usize, HotkeyChange),
    LinkPress(String),
    LinkOpened(bool), // true if success, false if failed
}
