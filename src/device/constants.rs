use uuid::Uuid;

/**
 * How often (milliseconds) to poll for new breath values / check for the connection status.
 */
pub const POLL_DELAY: u64 = 10;

/**
 * How often (milliseconds) to attempt to reconnect.
 */
pub const CONNECT_DELAY: u64 = 1000;

/**
 * How long (milliseconds) a write to a characteristic may take.
 */
pub const WRITE_DEADLINE: u64 = 2000;

/**
 * How long (milliseconds) checking if the peripheral is still connected may take
 */
pub const IS_CONNECTED_DEADLINE: u64 = 2000;

/**
 * The UUID of the Bluetooth BLE service for Melody Smart
 */
pub const MELODY_SMART_SERVICE: &str = "bc2f4cc6-aaef-4351-9034-d66268e328f0";

/**
 * The UUID of the Bluetooth BLE remote GATT characteristic to send data commands to.
 */
pub const MELODY_SMART_DATA_CHARACTERISTIC: &str = "06d1e5e7-79ad-4a71-8faa-373789f7d93c";
// pub const MELODY_SMART_COMMAND_CHARACTERISTIC: &str = "818ae306-9c5b-448d-b51a-7add6a5d314d";

pub const COMMAND_REQUEST_BREATH: [u8; 2] = [0x3F, 0x62]; // ?b
pub const COMMAND_LED_LEFT_ON: [u8; 2] = [0x6C, 0x31]; // l1
// pub const COMMAND_LED_LEFT_OFF: [u8; 2] = [0x6C, 0x30]; // l0
// pub const COMMAND_LED_RIGHT_ON: [u8; 2] = [0x72, 0x31]; // r1
// pub const COMMAND_LED_RIGHT_OFF: [u8; 2] = [0x72, 0x30]; // r0


/**
 * The range of the breath value received from the device.
 * - around BREATH_RANGE is neutral
 * - towards 0 represents strength of sipping
 * - towards BREATH_RANGE*2 represents strength of puffing.
 * Note that this value is normalized in most of the code by subtracting 2048 (so that 0 is neatral)
 */
pub const BREATH_RANGE: i16 = 2048;

pub fn make_melody_smart_service_uuid() -> Uuid {
    Uuid::parse_str(MELODY_SMART_SERVICE).unwrap()
}

pub fn make_melody_smart_data_uuid() -> Uuid {
    Uuid::parse_str(MELODY_SMART_DATA_CHARACTERISTIC).unwrap()
}
