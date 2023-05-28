use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use enum_iterator::{all, Sequence};

use crate::sim::types::Button;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence, Serialize, Deserialize)]
pub enum BreathDirection {
    Puff,
    Sip,
}

impl BreathDirection {
    pub fn all() -> Vec<BreathDirection> {
        all::<BreathDirection>().collect::<Vec<_>>()
    }
}

impl std::fmt::Display for BreathDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match self {
            BreathDirection::Puff => "Puff",
            BreathDirection::Sip => "Sip",
        };

        write!(f, "{}", result)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyConfig {
    pub breath_direction: BreathDirection,
    pub threshold: Option<i8>,
    pub modifier_shift: bool,
    pub modifier_ctrl: bool,
    pub modifier_meta: bool,
    pub modifier_alt: bool,
    pub button: Button,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub hotkeys: Vec<HotkeyConfig>,
}

impl Config {
    pub fn sort_hotkeys(&mut self) {
        self.hotkeys.sort_by(|a, b| {
            let a_sip = a.breath_direction == BreathDirection::Sip;

            if a.breath_direction != b.breath_direction {
                return if a_sip { Ordering::Less } else { Ordering::Greater };
            }

            a.threshold.unwrap_or(i8::max_value()).cmp(&b.threshold.unwrap_or(i8::max_value()))
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            hotkeys: vec![
                HotkeyConfig {
                    breath_direction: BreathDirection::Sip,
                    threshold: Some(30),
                    modifier_shift: false,
                    modifier_ctrl: false,
                    modifier_meta: false,
                    modifier_alt: false,
                    button: Button::MouseRight,
                },
                HotkeyConfig {
                    breath_direction: BreathDirection::Puff,
                    threshold: Some(30),
                    modifier_shift: false,
                    modifier_ctrl: false,
                    modifier_meta: false,
                    modifier_alt: false,
                    button: Button::MouseLeft,
                },
            ],
        }
    }
}
