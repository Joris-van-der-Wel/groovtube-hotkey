use futures::channel::mpsc::Sender;
use futures::SinkExt;
use iced::{Alignment, Application, Command, Element, Length, Settings, Size, Subscription, window};
use iced::event::{self, Event};
use iced::font::{self, Font};
use iced::time::{every as iced_time_every};
use iced::theme::{self, Theme};
use iced::widget::{
    Column, PickList, button, column, container, horizontal_rule, row, text, text_input, tooltip,
};
use iced::window::icon;
use iced::widget::tooltip::{Position as TooltipPosition};
use std::time::{Duration};
use log::{error, info};
use tokio_util::sync::{CancellationToken};

use crate::config::io::{ConfigIO};
use crate::config::types::{BREATH_DIRECTIONS, BreathDirection, Config, HotkeyConfig};
use crate::device::connection::connect_device_subscription;
use crate::device::types::{DeviceEvent, DeviceState};
use crate::error::{AppRunError, error_msgbox};
use crate::gui::executor::MyExecutor;
use crate::gui::open::open_link;
use crate::gui::style::{TextButtonStyleSheet};
use crate::gui::types::{Message, HotkeyChange, HotkeyModifier};
use crate::resources::{MUI_SYMBOLS_OUTLINED_BYTES, MUI_SYMBOLS_OUTLINED_FAMILY};
use crate::sim::breath_input_sim::breath_input_sim;
use crate::sim::types::{BreathInputSimCommand, Button as InputSimButton, BUTTONS as INPUT_SIM_BUTTONS};

const MUI_SYMBOLS_OUTLINED_FONT: Font = Font::with_name(MUI_SYMBOLS_OUTLINED_FAMILY);

pub struct ApplicationFlags {
    config_io: ConfigIO,
}

pub struct MyApplication {
    // this token is cancelled upon exit
    app_cancel: CancellationToken,

    // current config, might not be saved to disk yet
    config_io: ConfigIO,
    config: Config,
    config_dirty: bool,
    // this flag is used to make sure that a user is not spammed with save configuration errors
    displayed_config_save_error: bool,

    // Send events to this futures channel to simulate keyboard/mouse
    breath_input_sim_sender: (Sender<DeviceEvent>, Sender<BreathInputSimCommand>),

    // latest state from the device
    latest_device_state: DeviceState,
    latest_breath_value: i8,
}

impl MyApplication {
    fn before_close(&mut self) {
        self.app_cancel.cancel();
    }

    fn load_symbols_font(&self) -> Command<Message> {
        font::load(MUI_SYMBOLS_OUTLINED_BYTES).map(Message::SymbolsFontLoadComplete)
    }

    fn load_config(&self) -> Command<Message> {
        let mut sender = self.breath_input_sim_sender.1.clone();
        let config_io = self.config_io.clone();

        let fut = async move {
            let config = config_io.read().await.unwrap_or_else(|err| {
                if err.is_file_not_found_error() {
                    // this is probably the first start of the app
                    info!("Config file not found, using defaults");
                }
                else {
                    error_msgbox("Failed to load config", &err);
                }
                Config::default()
            });

            sender.send(BreathInputSimCommand::SetConfig(config.clone())).await
                .expect("Failed to send config to breath_input_sim");

            config
        };

        Command::perform(fut, Message::ConfigLoadComplete)
    }

    fn save_config(&self) -> Command<Message> {
        let displayed_config_save_error = self.displayed_config_save_error;
        let config = self.config.clone();
        let config_io = self.config_io.clone();

        let fut = async move {
            match config_io.save(config).await {
                Ok(_) => true,
                Err(err) => {
                    if displayed_config_save_error {
                        error!("Failed to save config: {:?}", err);
                    }
                    else {
                        error_msgbox("Failed to save config", &err);
                    }
                    false
                },
            }
        };

        return Command::perform(fut, Message::ConfigSaveComplete);
    }

    fn send_config(&self) -> Command<Message> {
        let mut sender = self.breath_input_sim_sender.1.clone();
        let config = self.config.clone();

        let fut = async move {
            sender.send(BreathInputSimCommand::SetConfig(config)).await
                .expect("Failed to send config to breath_input_sim");
        };

        Command::perform(fut, Message::WriteComplete)
    }

    fn open_link(&self, url: String) -> Command<Message> {
        let fut = async move {
            match open_link(&url).await {
                Ok(_) => true,
                Err(err) => {
                    error_msgbox("Failed to open link", &err);
                    false
                },
            }
        };

        return Command::perform(fut, Message::LinkOpened)
    }
}

impl Application for MyApplication {
    type Executor = MyExecutor;
    type Message = Message;
    type Theme = Theme;
    type Flags = ApplicationFlags;

    fn new(flags: ApplicationFlags) -> (MyApplication, Command<Self::Message>) {
        let app_cancel = CancellationToken::new();
        // todo: wait for the join handle of breath_input_sim when closing
        // todo: wait for device connection to be closed when closing
        let (bis_event_sender, bis_command_sender, _) = breath_input_sim(app_cancel.clone());

        let app = MyApplication {
            app_cancel,
            config_io: flags.config_io,
            config: Config::default(),
            config_dirty: false,
            displayed_config_save_error: false,
            breath_input_sim_sender: (bis_event_sender, bis_command_sender),
            latest_device_state: DeviceState::Initial,
            latest_breath_value: 0,
        };

        let command = Command::batch(vec![
            app.load_symbols_font(),
            app.load_config(),
        ]);
        (app, command)
    }

    fn title(&self) -> String {
        String::from(concat!("GroovTube Hotkey ", env!("CARGO_PKG_VERSION")))
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::SymbolsFontLoadComplete(result) => {
                result.expect("Failed to load symbols font");
                info!("Symbols font load complete");
            },
            Message::ConfigLoadComplete(config) => {
                info!("Config load complete");
                self.config = config;
            },
            Message::ApplyDirtyConfig => {
                if self.config_dirty {
                    self.config_dirty = false;

                    return Command::batch(vec![
                        self.send_config(),
                        self.save_config(),
                    ]);
                }
            },
            Message::ConfigSaveComplete(success) => {
                if !success {
                    self.displayed_config_save_error = true;
                }
            },

            Message::LinkPress(url) => {
                return self.open_link(url);
            },
            Message::EventOccurred(Event::Window(id, window::Event::CloseRequested)) => {
                info!("Close requested");
                self.before_close();
                return window::close(id);
            },
            Message::DeviceEvent(DeviceEvent::StateChange(state)) => {
                self.latest_device_state = state;
                self.latest_breath_value = 0;
            },
            Message::DeviceEvent(DeviceEvent::Breath(breath_value)) => {
                self.latest_breath_value = breath_value;
            },

            Message::AddHotkey => {
                self.config.hotkeys.push(HotkeyConfig {
                    breath_direction: BreathDirection::Puff,
                    threshold: None,
                    modifier_shift: false,
                    modifier_ctrl: false,
                    modifier_meta: false,
                    modifier_alt: false,
                    button: InputSimButton::MouseLeft,
                });
                self.config_dirty = true;
            },
            Message::HotkeyChange(index, change) => {
                if index < self.config.hotkeys.len() {
                    let config = &mut self.config.hotkeys[index];

                    match change {
                        HotkeyChange::BreathDirectionChange(direction) => {
                            config.breath_direction = direction;
                        },
                        HotkeyChange::ButtonChange(button) => {
                            config.button = button;
                        },
                        HotkeyChange::ThresholdChange(threshold_str) => {
                            if threshold_str.is_empty() {
                                config.threshold = None;
                            }
                            else {
                                if let Ok(strength) = threshold_str.parse::<i8>() {
                                    config.threshold = Some(strength.clamp(1, 99));
                                }
                                // ignore parse error, in which case the value is not changed
                            }
                        },
                        HotkeyChange::ModifierToggle(HotkeyModifier::Shift) => {
                            config.modifier_shift = !config.modifier_shift;
                        },
                        HotkeyChange::ModifierToggle(HotkeyModifier::Ctrl) => {
                            config.modifier_ctrl = !config.modifier_ctrl;
                        },
                        HotkeyChange::ModifierToggle(HotkeyModifier::Meta) => {
                            config.modifier_meta = !config.modifier_meta;
                        },
                        HotkeyChange::ModifierToggle(HotkeyModifier::Alt) => {
                            config.modifier_alt = !config.modifier_alt;
                        },
                        HotkeyChange::Delete => {
                            self.config.hotkeys.remove(index);
                        },
                    }

                    self.config_dirty = true;
                }
            },

            _ => {}
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            event::listen().map(Message::EventOccurred),
            iced_time_every(Duration::from_secs(1)).map(|_| Message::ApplyDirtyConfig),
            connect_device_subscription(
                self.app_cancel.clone(),
                vec![self.breath_input_sim_sender.0.clone()],
            ).map(Message::DeviceEvent)
        ])
    }

    fn view(&self) -> Element<Message> {
        let modifier_toggle = |
            description: &'static str,
            symbol: char,
            checked: bool,
            on_press: Message,
        | -> Element<Message> {
            tooltip(
                button(
                    text(symbol.to_string()).font(MUI_SYMBOLS_OUTLINED_FONT)
                )
                    .style(if checked { theme::Button::Primary } else { theme::Button::Secondary })
                    .on_press(on_press),
                description,
                TooltipPosition::Bottom,
            ).into()
        };

        let hotkey_form = |index: usize, config: &HotkeyConfig| -> Element<Message> {
            let threshold_value = match config.threshold {
                None => "".to_string(),
                Some(value) => value.to_string(),
            };

            row![
                PickList::new(
                    BREATH_DIRECTIONS,
                    Some(config.breath_direction),
                    move |value| Message::HotkeyChange(index, HotkeyChange::BreathDirectionChange(value)),
                ).width(60),

                row![
                    text("≥"),
                    text_input("", threshold_value.as_str())
                        .width(30)
                        .on_input(move |value| Message::HotkeyChange(index, HotkeyChange::ThresholdChange(value))),
                    text("%"),
                ].align_items(Alignment::Center).spacing(2),

                row![
                    modifier_toggle("Shift", '\u{e5f2}', config.modifier_shift, Message::HotkeyChange(index, HotkeyChange::ModifierToggle(HotkeyModifier::Shift))),
                    modifier_toggle("Ctrl", '\u{eae6}', config.modifier_ctrl, Message::HotkeyChange(index, HotkeyChange::ModifierToggle(HotkeyModifier::Ctrl))),
                    modifier_toggle("Meta", '\u{eae7}', config.modifier_meta, Message::HotkeyChange(index, HotkeyChange::ModifierToggle(HotkeyModifier::Meta))),
                    modifier_toggle("Alt", '\u{eae8}', config.modifier_alt, Message::HotkeyChange(index, HotkeyChange::ModifierToggle(HotkeyModifier::Alt))),
                ].spacing(2),

                PickList::new(
                    INPUT_SIM_BUTTONS,
                    Some(config.button),
                    move |value| Message::HotkeyChange(index, HotkeyChange::ButtonChange(value)),
                ).width(200),

                button(
                    text("\u{e92b}").font(MUI_SYMBOLS_OUTLINED_FONT)
                )
                .style(theme::Button::Destructive)
                .on_press(Message::HotkeyChange(index, HotkeyChange::Delete)),
            ]
            .align_items(Alignment::Center)
            .spacing(20)
            .into()
        };

        let mut add_hotkey_button = button(
            text("\u{e147}").font(MUI_SYMBOLS_OUTLINED_FONT)
        )
        .style(theme::Button::Positive);

        if self.config.hotkeys.len() < 8 {
            add_hotkey_button = add_hotkey_button.on_press(Message::AddHotkey);
        }

        let device_state = match self.latest_device_state {
            DeviceState::Initial => "".to_string(),
            DeviceState::Scanning { no_permission: false } => "Scanning…".to_string(),
            DeviceState::Scanning { no_permission: true } => "Not allowed to access Bluetooth!".to_string(),
            DeviceState::Connecting => "Connecting…".to_string(),
            DeviceState::Connected => {
                let percentage = self.latest_breath_value;
                if percentage < 0 {
                    format!("{}% sip", -percentage)
                }
                else {
                    format!("{}% puff", percentage)
                }
            },
        };

        container(
            column![
                column![
                    text(device_state),

                    horizontal_rule(10),

                    Column::with_children(
                        self.config.hotkeys
                            .iter()
                            .enumerate()
                            .map(|(index, config)| hotkey_form(index, config))
                            .map(Element::from)
                    )
                        .spacing(30)
                        .width(Length::Shrink),

                    column![add_hotkey_button]
                        .align_items(Alignment::Center)
                        .width(Length::Fill)
                        .spacing(20),
                ]
                    .spacing(30)
                    .width(Length::Fill)
                    .align_items(Alignment::Center)
                    .height(Length::Fill),

                button(
                    text("github.com/Joris-van-der-Wel/groovtube-hotkey")
                        .size(14)
                )
                    .style(theme::Button::Custom(Box::new(TextButtonStyleSheet)))
                    .on_press(Message::LinkPress("https://github.com/Joris-van-der-Wel/groovtube-hotkey".to_string())),

            ].align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .padding(20)
        .into()
    }
}

fn make_icon() -> icon::Icon {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/icon-32-rgba"));
    let bytes = bytes.to_vec();
    icon::from_rgba(bytes, 32, 32).expect("Failed to load window icon")
}

pub fn run_application() -> Result<(), AppRunError> {
    let mut config_io = ConfigIO::new_sync()?;
    let mut config_locker = config_io.locker()?;
    let _lock_guard = config_locker.lock()?;

    let flags = ApplicationFlags { config_io };
    let mut settings = Settings::with_flags(flags);

    // handle exits ourselves (Event::CloseRequested)
    settings.id = Some("groovtube-hotkey".to_string());
    settings.window.exit_on_close_request = false;
    settings.window.size = Size::new(600.0, 700.0);
    settings.window.resizable = false;
    settings.window.icon = Some(make_icon());

    // This is no longer necessary because iced is forced to use the software renderer (by removing
    // the wgpu feature).
    // However should the wgpu feature be enabled again in the future, make sure to test this
    // program with old crappy integrated intel graphics. The following lines force
    // DirectX 12 to be used instead of Vulkan on windows, vulkan appears to be very buggy on old
    // intel CPUs.
    // #[cfg(target_os = "windows")]
    // std::env::set_var("WGPU_BACKEND", "dx12");

    // this function will call process::exit() unless there was a startup error
    MyApplication::run(settings)?;
    Ok(())
}
