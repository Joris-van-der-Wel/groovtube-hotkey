#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceState {
    Initial,
    Scanning { no_permission: bool },
    Connecting,
    Connected,
}

#[derive(Debug, Clone)]
pub enum DeviceEvent {
    StateChange(DeviceState),
    Breath(i8), // [-100, 100]
}
