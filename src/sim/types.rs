use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use rdev;
use enum_iterator::{all, Sequence};

use crate::config::types::Config;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Sequence)]
// Sorted in a way that sorta makes sense for display in a dropdown
pub enum Button {
    MouseLeft,
    MouseRight,
    MouseMiddle,

    UpArrow,
    RightArrow,
    DownArrow,
    LeftArrow,
    ShiftLeft,
    ShiftRight,
    Function,
    Escape,
    Tab,
    Backspace,
    Return,
    CapsLock,
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Space,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    BackQuote,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Minus,
    Equal,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Alt,
    AltGr,
    ControlLeft,
    ControlRight,
    MetaLeft,
    MetaRight,
    BackSlash,
    Comma,
    Dot,
    LeftBracket,
    Quote,
    RightBracket,
    SemiColon,
    Slash,
    PrintScreen,
    ScrollLock,
    Pause,
    NumLock,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDelete,
    KpDivide,
    KpMinus,
    KpMultiply,
    KpPlus,
    KpReturn,
}

impl Button {
    pub fn all() -> Vec<Button> {
        all::<Button>().collect::<Vec<_>>()
    }

    pub fn rdev_mouse_button(&self) -> Option<rdev::Button> {
        match self {
            Button::MouseLeft => Some(rdev::Button::Left),
            Button::MouseRight => Some(rdev::Button::Right),
            Button::MouseMiddle => Some(rdev::Button::Middle),
            _ => None,
        }
    }

    pub fn rdev_key(&self) -> Option<rdev::Key> {
        match self {
            Button::MouseLeft | Button::MouseRight | Button::MouseMiddle => None,

            Button::UpArrow => Some(rdev::Key::UpArrow),
            Button::RightArrow => Some(rdev::Key::RightArrow),
            Button::DownArrow => Some(rdev::Key::DownArrow),
            Button::LeftArrow => Some(rdev::Key::LeftArrow),
            Button::ShiftLeft => Some(rdev::Key::ShiftLeft),
            Button::ShiftRight => Some(rdev::Key::ShiftRight),
            Button::Function => Some(rdev::Key::Function),
            Button::Escape => Some(rdev::Key::Escape),
            Button::Tab => Some(rdev::Key::Tab),
            Button::Backspace => Some(rdev::Key::Backspace),
            Button::Return => Some(rdev::Key::Return),
            Button::CapsLock => Some(rdev::Key::CapsLock),
            Button::Insert => Some(rdev::Key::Insert),
            Button::Delete => Some(rdev::Key::Delete),
            Button::Home => Some(rdev::Key::Home),
            Button::End => Some(rdev::Key::End),
            Button::PageUp => Some(rdev::Key::PageUp),
            Button::PageDown => Some(rdev::Key::PageDown),
            Button::Space => Some(rdev::Key::Space),
            Button::KeyA => Some(rdev::Key::KeyA),
            Button::KeyB => Some(rdev::Key::KeyB),
            Button::KeyC => Some(rdev::Key::KeyC),
            Button::KeyD => Some(rdev::Key::KeyD),
            Button::KeyE => Some(rdev::Key::KeyE),
            Button::KeyF => Some(rdev::Key::KeyF),
            Button::KeyG => Some(rdev::Key::KeyG),
            Button::KeyH => Some(rdev::Key::KeyH),
            Button::KeyI => Some(rdev::Key::KeyI),
            Button::KeyJ => Some(rdev::Key::KeyJ),
            Button::KeyK => Some(rdev::Key::KeyK),
            Button::KeyL => Some(rdev::Key::KeyL),
            Button::KeyM => Some(rdev::Key::KeyM),
            Button::KeyN => Some(rdev::Key::KeyN),
            Button::KeyO => Some(rdev::Key::KeyO),
            Button::KeyP => Some(rdev::Key::KeyP),
            Button::KeyQ => Some(rdev::Key::KeyQ),
            Button::KeyR => Some(rdev::Key::KeyR),
            Button::KeyS => Some(rdev::Key::KeyS),
            Button::KeyT => Some(rdev::Key::KeyT),
            Button::KeyU => Some(rdev::Key::KeyU),
            Button::KeyV => Some(rdev::Key::KeyV),
            Button::KeyW => Some(rdev::Key::KeyW),
            Button::KeyX => Some(rdev::Key::KeyX),
            Button::KeyY => Some(rdev::Key::KeyY),
            Button::KeyZ => Some(rdev::Key::KeyZ),
            Button::BackQuote => Some(rdev::Key::BackQuote),
            Button::Num0 => Some(rdev::Key::Num0),
            Button::Num1 => Some(rdev::Key::Num1),
            Button::Num2 => Some(rdev::Key::Num2),
            Button::Num3 => Some(rdev::Key::Num3),
            Button::Num4 => Some(rdev::Key::Num4),
            Button::Num5 => Some(rdev::Key::Num5),
            Button::Num6 => Some(rdev::Key::Num6),
            Button::Num7 => Some(rdev::Key::Num7),
            Button::Num8 => Some(rdev::Key::Num8),
            Button::Num9 => Some(rdev::Key::Num9),
            Button::Minus => Some(rdev::Key::Minus),
            Button::Equal => Some(rdev::Key::Equal),
            Button::F1 => Some(rdev::Key::F1),
            Button::F2 => Some(rdev::Key::F2),
            Button::F3 => Some(rdev::Key::F3),
            Button::F4 => Some(rdev::Key::F4),
            Button::F5 => Some(rdev::Key::F5),
            Button::F6 => Some(rdev::Key::F6),
            Button::F7 => Some(rdev::Key::F7),
            Button::F8 => Some(rdev::Key::F8),
            Button::F9 => Some(rdev::Key::F9),
            Button::F10 => Some(rdev::Key::F10),
            Button::F11 => Some(rdev::Key::F11),
            Button::F12 => Some(rdev::Key::F12),
            Button::Alt => Some(rdev::Key::Alt),
            Button::AltGr => Some(rdev::Key::AltGr),
            Button::ControlLeft => Some(rdev::Key::ControlLeft),
            Button::ControlRight => Some(rdev::Key::ControlRight),
            Button::MetaLeft => Some(rdev::Key::MetaLeft),
            Button::MetaRight => Some(rdev::Key::MetaRight),
            Button::BackSlash => Some(rdev::Key::BackSlash),
            Button::Comma => Some(rdev::Key::Comma),
            Button::Dot => Some(rdev::Key::Dot),
            Button::LeftBracket => Some(rdev::Key::LeftBracket),
            Button::Quote => Some(rdev::Key::Quote),
            Button::RightBracket => Some(rdev::Key::RightBracket),
            Button::SemiColon => Some(rdev::Key::SemiColon),
            Button::Slash => Some(rdev::Key::Slash),
            Button::PrintScreen => Some(rdev::Key::PrintScreen),
            Button::ScrollLock => Some(rdev::Key::ScrollLock),
            Button::Pause => Some(rdev::Key::Pause),
            Button::NumLock => Some(rdev::Key::NumLock),
            Button::Kp0 => Some(rdev::Key::Kp0),
            Button::Kp1 => Some(rdev::Key::Kp1),
            Button::Kp2 => Some(rdev::Key::Kp2),
            Button::Kp3 => Some(rdev::Key::Kp3),
            Button::Kp4 => Some(rdev::Key::Kp4),
            Button::Kp5 => Some(rdev::Key::Kp5),
            Button::Kp6 => Some(rdev::Key::Kp6),
            Button::Kp7 => Some(rdev::Key::Kp7),
            Button::Kp8 => Some(rdev::Key::Kp8),
            Button::Kp9 => Some(rdev::Key::Kp9),
            Button::KpDelete => Some(rdev::Key::KpDelete),
            Button::KpDivide => Some(rdev::Key::KpDivide),
            Button::KpMinus => Some(rdev::Key::KpMinus),
            Button::KpMultiply => Some(rdev::Key::KpMultiply),
            Button::KpPlus => Some(rdev::Key::KpPlus),
            Button::KpReturn => Some(rdev::Key::KpReturn),
        }
    }
}


impl std::fmt::Display for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let result = match self {
            Button::MouseLeft => "Mouse Left",
            Button::MouseRight => "Mouse Right",
            Button::MouseMiddle => "Mouse Middle",

            Button::Alt => "Alt",
            Button::AltGr => "AltGr",
            Button::BackQuote => "`",
            Button::BackSlash => "\\",
            Button::Backspace => "Backspace",
            Button::CapsLock => "CapsLock",
            Button::Comma => ",",
            Button::ControlLeft => "Control Left",
            Button::ControlRight => "Control Right",
            Button::Delete => "Delete",
            Button::Dot => ".",
            Button::DownArrow => "DownArrow",
            Button::End => "End",
            Button::Equal => "=",
            Button::Escape => "Esc",
            Button::F1 => "F1",
            Button::F2 => "F2",
            Button::F3 => "F3",
            Button::F4 => "F4",
            Button::F5 => "F5",
            Button::F6 => "F6",
            Button::F7 => "F7",
            Button::F8 => "F8",
            Button::F9 => "F9",
            Button::F10 => "F10",
            Button::F11 => "F11",
            Button::F12 => "F12",
            Button::Function => "Fn",
            Button::Home => "Home",
            Button::Insert => "Insert",
            Button::KeyA => "A",
            Button::KeyB => "B",
            Button::KeyC => "C",
            Button::KeyD => "D",
            Button::KeyE => "E",
            Button::KeyF => "F",
            Button::KeyG => "G",
            Button::KeyH => "H",
            Button::KeyI => "I",
            Button::KeyJ => "J",
            Button::KeyK => "K",
            Button::KeyL => "L",
            Button::KeyM => "M",
            Button::KeyN => "N",
            Button::KeyO => "O",
            Button::KeyP => "P",
            Button::KeyQ => "Q",
            Button::KeyR => "R",
            Button::KeyS => "S",
            Button::KeyT => "T",
            Button::KeyU => "U",
            Button::KeyV => "V",
            Button::KeyW => "W",
            Button::KeyX => "X",
            Button::KeyY => "Y",
            Button::KeyZ => "Z",
            Button::Kp0 => "Keypad 0",
            Button::Kp1 => "Keypad 1",
            Button::Kp2 => "Keypad 2",
            Button::Kp3 => "Keypad 3",
            Button::Kp4 => "Keypad 4",
            Button::Kp5 => "Keypad 5",
            Button::Kp6 => "Keypad 6",
            Button::Kp7 => "Keypad 7",
            Button::Kp8 => "Keypad 8",
            Button::Kp9 => "Keypad 9",
            Button::KpDelete => "Keypad Delete",
            Button::KpDivide => "Keypad /",
            Button::KpMinus => "Keypad -",
            Button::KpMultiply => "Keypad *",
            Button::KpPlus => "Keypad +",
            Button::KpReturn => "Keypad Return",
            Button::LeftArrow => "Left Arrow",
            Button::LeftBracket => "Left Bracket",
            Button::MetaLeft => "Meta Left",
            Button::MetaRight => "Meta Right",
            Button::Minus => "-",
            Button::Num0 => "0",
            Button::Num1 => "1",
            Button::Num2 => "2",
            Button::Num3 => "3",
            Button::Num4 => "4",
            Button::Num5 => "5",
            Button::Num6 => "6",
            Button::Num7 => "7",
            Button::Num8 => "8",
            Button::Num9 => "9",
            Button::NumLock => "Num Lock",
            Button::PageDown => "Page Down",
            Button::PageUp => "Page Up",
            Button::Pause => "Pause",
            Button::PrintScreen => "Print Screen",
            Button::Quote => "'",
            Button::Return => "Return",
            Button::RightArrow => "Right Arrow",
            Button::RightBracket => "Right Bracket",
            Button::ScrollLock => "Scroll Lock",
            Button::SemiColon => ";",
            Button::ShiftLeft => "Shift Left",
            Button::ShiftRight => "Shift Right",
            Button::Slash => "/",
            Button::Space => "Space",
            Button::Tab => "Tab",
            Button::UpArrow => "Up Arrow",
        };

        write!(f, "{}", result)
    }
}

pub type HeldButtons = IndexSet<Button>;

pub enum InputSimCommand {
    SetHeldButtons(HeldButtons),
}

pub enum BreathInputSimCommand {
    SetConfig(Config),
}
