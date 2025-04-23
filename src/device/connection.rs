use std::convert::Infallible;
use std::error::Error;
use iced::subscription::{self, Subscription};
use futures::{StreamExt, SinkExt};
use futures::channel::mpsc::Sender;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};
use log::{debug, info, warn};
use tokio::spawn;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tokio::time::{sleep, Duration};

use crate::device::constants::{make_melody_smart_service_uuid, make_melody_smart_data_uuid, CONNECT_DELAY, POLL_DELAY, COMMAND_REQUEST_BREATH, BREATH_RANGE, IS_CONNECTED_DEADLINE, WRITE_DEADLINE, COMMAND_LED_LEFT_ON};
use crate::device::types::{DeviceEvent, DeviceState};
use crate::error::DeviceError;

#[derive(Debug)]
enum ConnectionState {
    Scanning {
        retry: bool,
        no_permission: bool,
        adapters: Option<Vec<Adapter>>,
    },
    Connecting {
        peripheral: Peripheral,
    },
    Connected {
        peripheral: Peripheral,
        data_char: Characteristic,
    },
}

async fn start_scanning(manager: &Manager) -> Result<Vec<Adapter>, DeviceError> {
    let adapters = manager.adapters().await?;
    let melody_smart_service_uuid = make_melody_smart_service_uuid();

    let filter = ScanFilter {
        services: vec![melody_smart_service_uuid],
    };

    for adapter in &adapters {
        info!("Scanning using adapter {}...", adapter.adapter_info().await.unwrap_or("UNKNOWN".to_string()));
        adapter.start_scan(filter.clone()).await?;
    }

    Ok(adapters)
}

async fn find_peripheral(adapters: &Vec<Adapter>) -> Result<Option<Peripheral>, DeviceError> {
    let melody_smart_service_uuid = make_melody_smart_service_uuid();

    for adapter in adapters {
        let peripherals = match adapter.peripherals().await {
            Ok(v) => v,
            Err(err) => {
                warn!("Failed to query BLE adapter for peripherals: {}", err);
                continue;
            },
        };

        for peripheral in peripherals {
            let properties = peripheral.properties().await;

            match properties {
                Err(err) => {
                    warn!("Could not query peripheral for properties: {:?}", err);
                },
                Ok(None) => {
                    warn!("Peripheral has no properties");
                },
                Ok(Some(properties)) => {
                    // Some environments ignore the filter, so make sure to check the service uuid again
                    if properties.services.contains(&melody_smart_service_uuid) {
                        info!(
                            "Using peripheral {} {:?} {} {:?}",
                            properties.address,
                            properties.address_type,
                            properties.local_name.unwrap_or(String::from("NONE")),
                            properties.services,
                        );
                        return Ok(Some(peripheral));
                    }

                }
            }
        }
    }

    Ok(None)
}

async fn connect_peripheral(peripheral: &Peripheral) -> Result<Characteristic, DeviceError> {
    let melody_smart_service_uuid = make_melody_smart_service_uuid();
    let melody_smart_data_uuid = make_melody_smart_data_uuid();

    info!("Connecting to peripheral...");
    peripheral.connect().await?;

    info!("Connected; Discovering services...");
    peripheral.discover_services().await?;

    for service in peripheral.services() {
        if !service.uuid.eq(&melody_smart_service_uuid) {
            continue;
        }

        for characteristic in &service.characteristics {
            if !characteristic.uuid.eq(&melody_smart_data_uuid) {
                continue;
            }

            info!("Subscribing to characteristic {:?} {:?}", service.uuid, characteristic.uuid);
            peripheral.subscribe(&characteristic).await?;
            return Ok(characteristic.clone());
        }
    }

    Err(DeviceError::MissingCharacteristic)
}

async fn advance_state(state: ConnectionState, manager: &Manager) -> ConnectionState {
    match state {
        ConnectionState::Scanning { adapters, retry, .. } => {
            if retry {
                sleep(Duration::from_millis(CONNECT_DELAY)).await;
            }

            let adapters = match adapters {
                None => {
                    match start_scanning(&manager).await {
                        Ok(adapters) => Some(adapters),
                        Err(err) => {
                            warn!("Scanning failed {:?}", err);

                            let mut no_permission_error = false;
                            if let Some(source) = err.source() {
                                if let Some(btleplug::Error::PermissionDenied) = source.downcast_ref::<btleplug::Error>() {
                                    no_permission_error = true;
                                }
                            }

                            return ConnectionState::Scanning { adapters: None, retry: true, no_permission: no_permission_error };
                        },
                    }
                },
                Some(adapters) => Some(adapters),
            };

            match find_peripheral(adapters.as_ref().unwrap()).await {
                Ok(Some(peripheral)) => {
                    ConnectionState::Connecting { peripheral }
                },
                Ok(None) => {
                    debug!("No peripherals matched");
                    ConnectionState::Scanning { adapters, retry: true, no_permission: false }
                },
                Err(err) => {
                    warn!("Finding peripheral failed: {:?}", err);
                    ConnectionState::Scanning { adapters, retry: true, no_permission: false }
                },
            }
        },
        ConnectionState::Connecting { peripheral } => {
            let data_char = match connect_peripheral(&peripheral).await {
                Ok(v) => v,
                Err(err) => {
                    warn!("Connecting to peripheral failed: {:?}", err);
                    // If a peripheral fails to connect it might be because of the error:
                    //   Btle { source: Other("Error { code: HRESULT(0x80000013), message: \"The object has been closed.\" }") }
                    // In which case we have to obtain a new Peripheral. So go back to the scanning state
                    return ConnectionState::Scanning { adapters: None, retry: true, no_permission: false };
                },
            };

            send_led_left_on(&peripheral, &data_char).await;
            
            info!("Peripheral ready");
            ConnectionState::Connected { peripheral, data_char }
        },
        ConnectionState::Connected { peripheral, data_char } => {
            tokio::select! {
                _ = sleep(Duration::from_millis(IS_CONNECTED_DEADLINE)) => {
                    // macOS
                    warn!("Checking for connection status took too long");
                    sleep(Duration::from_millis(CONNECT_DELAY)).await;
                    ConnectionState::Scanning { adapters: None, retry: true, no_permission: false }
                }
                result = peripheral.is_connected() => match result {
                    Err(err) => {
                        warn!("Error checking for connection state: {:?}", err);
                        sleep(Duration::from_millis(CONNECT_DELAY)).await;
                        ConnectionState::Scanning { adapters: None, retry: true, no_permission: false }
                    },
                    Ok(false) => {
                        warn!("Connection lost");
                        sleep(Duration::from_millis(CONNECT_DELAY)).await;
                        ConnectionState::Scanning { adapters: None, retry: true, no_permission: false }
                    },
                    Ok(true) => ConnectionState::Connected { peripheral, data_char },
                }
            }
        },
    }
}

async fn send_led_left_on(peripheral: &Peripheral, data_char: &Characteristic) {
    let fut = peripheral.write(&data_char, &COMMAND_LED_LEFT_ON, WriteType::WithResponse);

    tokio::select! {
        _ = sleep(Duration::from_millis(WRITE_DEADLINE)) => {
            warn!("Sending to data characteristic took too long");
        }
        result = fut => {
            if let Err(err) = result {
                warn!("Failed to send to data characteristic: {:?}", err);
            }
        }
    };
}

async fn request_breath(peripheral: &Peripheral, data_char: &Characteristic) {
    let fut = peripheral.write(&data_char, &COMMAND_REQUEST_BREATH, WriteType::WithResponse);

    tokio::select! {
        _ = sleep(Duration::from_millis(WRITE_DEADLINE)) => {
            warn!("Sending to data characteristic took too long");
        }
        result = fut => {
            if let Err(err) = result {
                warn!("Failed to send to data characteristic: {:?}", err);
            }
        }
    };
}

fn read_notifications_task(cancel: CancellationToken, peripheral: &Peripheral, mut senders: Vec<Sender<DeviceEvent>>) -> JoinHandle<Result<(), DeviceError>> {
    let peripheral_clone = peripheral.clone();
    let melody_smart_data_uuid = make_melody_smart_data_uuid();

    return spawn(async move {
        let mut notification_stream = peripheral_clone.notifications().await?;
        let mut previous_value: i8 = 0;

        'mainloop: loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    break 'mainloop;
                },
                Some(data) = notification_stream.next() => {
                    if data.uuid.eq(&melody_smart_data_uuid) {
                        // this is a reply to COMMAND_REQUEST_BREATH
                        let str = String::from_utf8_lossy(data.value.as_slice());

                        match u16::from_str_radix(&str, 16) {
                            Err(err) => warn!("Failed to decode breath value {:?}", err),
                            Ok(parsed) => {
                                let range = BREATH_RANGE as f32;

                                // convert the breath range to a value between -100 and 100
                                let value = f32::try_from(parsed)
                                    .unwrap_or(range) // default to neutral (2048)
                                    .clamp(0.0_f32, range * 2.0_f32); // clamp to [0, 4096]
                                let value =
                                    (value - range) // normalize to [-2048, 2048]; 0 is now neutral
                                    / range // convert to a factor: [-1, 1]
                                    * 100.0_f32; // convert to a percentage
                                let value = value
                                    .round() as i8; // convert to an integer number, this also helps to avoid unnecessary updates

                                if previous_value != value {
                                    previous_value = value;
                                    for sender in &mut senders {
                                        sender.send(DeviceEvent::Breath(previous_value)).await.expect("Failed to send DeviceEvent")
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    });
}

async fn connect_device(cancel: CancellationToken, mut senders: Vec<Sender<DeviceEvent>>) -> Infallible {
    let mut connection_state = Some(ConnectionState::Scanning { adapters: None, retry: false, no_permission: false });
    let mut previous_device_state: Option<DeviceState> = None;
    let mut read_notifications_task_handle: Option<JoinHandle<Result<(), DeviceError>>> = None;
    let manager = Manager::new().await.unwrap();
    let mut connection_cancel = cancel.child_token();

    // note: subscription::channel expects the future to never resolve (Infallible)
    // so this loop is not stopped if `cancel` is cancelled.
    loop {
        let new_connection_state = advance_state(connection_state.take().unwrap(), &manager).await;

        let device_state = match &new_connection_state {
            ConnectionState::Scanning { no_permission, .. } => DeviceState::Scanning {
                no_permission: *no_permission,
            },
            ConnectionState::Connecting { .. } => DeviceState::Connecting,
            ConnectionState::Connected { .. } => DeviceState::Connected,
        };

        if previous_device_state.is_none() || previous_device_state.as_ref().unwrap() != &device_state {
            for sender in &mut senders {
                let event = DeviceEvent::StateChange(device_state.clone());
                sender.send(event).await.expect("Failed to send DeviceEvent")
            }

            previous_device_state = Some(device_state);
        }

        connection_state = Some(new_connection_state);

        match &connection_state {
            Some(ConnectionState::Connected { peripheral, data_char }) => {
                // Connected, start task to read notifications if not already started
                // and send ?b commands to the device every 10ms
                read_notifications_task_handle.get_or_insert_with(
                    || read_notifications_task(connection_cancel.clone(), peripheral, senders.clone())
                );
                request_breath(peripheral, data_char).await;
            },
            _ => {
                // Not connected, cancel and join the read notifications task if we have not already done so
                connection_cancel.cancel();
                connection_cancel = CancellationToken::new();

                if let Some(handle) = read_notifications_task_handle.take() {
                    info!("Waiting for read notifications task to stop");
                    handle.await
                        .expect("Failed to join read notifications task")
                        .expect("Error during read notifications task");
                    info!("Read notifications task stopped")
                }
            },
        }

        sleep(Duration::from_millis(POLL_DELAY)).await;
    }
}

pub fn connect_device_subscription(cancel: CancellationToken, senders: Vec<Sender<DeviceEvent>>) -> Subscription<DeviceEvent> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        64,
        move |subscription_sender| {
            let cancel2 = cancel.clone();
            let mut senders2 = senders.clone();
            senders2.push(subscription_sender);

            async move {
                connect_device(cancel2, senders2).await
            }
        },
    )
}
