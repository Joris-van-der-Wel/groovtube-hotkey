use iced::{Event};

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
    ConfigLoadComplete(Config),
    ConfigSaveComplete(bool), // true if success, false if failed
    DeviceEvent(DeviceEvent),
    AddHotkey,
    HotkeyChange(usize, HotkeyChange),
}
