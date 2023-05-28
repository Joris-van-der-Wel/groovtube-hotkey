use indexmap::IndexSet;
use tokio::spawn;
use tokio::task::JoinHandle;
use futures::StreamExt;
use futures::channel::mpsc::{channel, Sender};
use tokio_util::sync::CancellationToken;
use rdev::{EventType, simulate};
use crate::sim::types::{HeldButtons, InputSimCommand};

fn send(event_type: &EventType) {
    if let Err(err) = simulate(event_type) {
        println!("Failed to simulate {:?}: {:?}", event_type, err);
    }
}

pub fn input_sim_task(cancel: CancellationToken) -> (Sender<InputSimCommand>, JoinHandle<()>) {
    let (tx, mut rx) = channel::<InputSimCommand>(128);

    let handle = spawn(async move {
        let mut held_buttons: HeldButtons = IndexSet::new();

        'mainloop: loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    break 'mainloop;
                },
                Some(command) = rx.next() => {
                    match command {
                        InputSimCommand::SetHeldButtons(new_buttons) => {
                            for button in held_buttons.difference(&new_buttons) {
                                if let Some(btn) = button.rdev_mouse_button() {
                                    send(&EventType::ButtonRelease(btn));
                                }
                                else if let Some(key) = button.rdev_key() {
                                    send(&EventType::KeyRelease(key));
                                }
                            }

                            for button in new_buttons.difference(&held_buttons) {
                                if let Some(btn) = button.rdev_mouse_button() {
                                    send(&EventType::ButtonPress(btn));
                                }
                                else if let Some(key) = button.rdev_key() {
                                    send(&EventType::KeyPress(key));
                                }
                            }

                            held_buttons = new_buttons;
                        },
                    }
                },
            }
        }
    });

    return (tx, handle);
}