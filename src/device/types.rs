#[derive(Debug, Clone)]
pub enum DeviceState {
    Initial,
    Scanning,
    Connecting,
    Connected,
}

#[derive(Debug, Clone)]
pub enum DeviceEvent {
    StateChange(DeviceState),
    Breath(i8), // [-100, 100]
}
