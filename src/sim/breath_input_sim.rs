use indexmap::IndexSet;
use tokio::spawn;
use tokio::task::JoinHandle;
use futures::channel::mpsc::{channel, Sender};
use tokio_util::sync::CancellationToken;
use futures::{StreamExt, SinkExt};

use crate::config::types::{HotkeyConfig, BreathDirection};
use crate::device::types::DeviceEvent;
use crate::sim::input_sim::input_sim_task;
use crate::sim::types::{HeldButtons, BreathInputSimCommand, InputSimCommand, Button};

pub fn breath_input_sim(cancel: CancellationToken) -> (Sender<DeviceEvent>, Sender<BreathInputSimCommand>, JoinHandle<()>) {
    let (event_sender, mut event_receiver) = channel::<DeviceEvent>(128);
    let (command_sender, mut command_receiver) = channel::<BreathInputSimCommand>(8);
    let (mut input_sim_tx, input_sim_handle) = input_sim_task(cancel.clone());

    let handle = spawn(async move {
        let mut puff_hotkeys: Vec<HotkeyConfig> = Vec::new();
        let mut sip_hotkeys: Vec<HotkeyConfig> = Vec::new();

        'mainloop: loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    break 'mainloop;
                },
                Some(event) = event_receiver.next() => {
                    if let DeviceEvent::Breath(breath_value) = event {
                        let hotkeys = if breath_value < 0 { &sip_hotkeys } else { & puff_hotkeys };

                        let breath_value_abs = breath_value.abs();

                        let hotkey = hotkeys.into_iter().find(|hotkey| {
                            if let Some(threshold) = hotkey.threshold {
                                if breath_value_abs >= threshold {
                                    return true;
                                }
                            }
                            false
                        });

                        let mut buttons: HeldButtons = IndexSet::new();

                        if let Some(hotkey) = hotkey {
                            if hotkey.modifier_shift { buttons.insert(Button::ShiftLeft); }
                            if hotkey.modifier_ctrl { buttons.insert(Button::ControlLeft); }
                            if hotkey.modifier_meta { buttons.insert(Button::MetaLeft); }
                            if hotkey.modifier_alt { buttons.insert(Button::Alt); }
                            buttons.insert(hotkey.button);
                        }
                        // if no hotkey matched, send an empty HeldButtons, so that all buttons will be released

                        input_sim_tx.send(InputSimCommand::SetHeldButtons(buttons))
                            .await
                            .expect("Failed to send command to input_sim: {:?}");
                    }
                },
                Some(command) = command_receiver.next() => {
                    match command {
                        BreathInputSimCommand::SetConfig(new_config) => {
                            let mut hotkeys = new_config.hotkeys;
                            // sort descending by threshold, None last
                            hotkeys.sort_by(
                                |a, b|
                                b.threshold.unwrap_or(0).cmp(&a.threshold.unwrap_or(0))
                            );

                            (puff_hotkeys, sip_hotkeys) = hotkeys
                                .into_iter()
                                .partition(|hotkey| hotkey.breath_direction == BreathDirection::Puff);
                        },
                    }
                },
            }
        }

        input_sim_handle.await.expect("Failed to join input_sim_task");

        ()
    });

    return (event_sender, command_sender, handle);
}