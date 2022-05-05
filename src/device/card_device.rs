use std::default::default;

use serde::Serialize;

use super::{
    a2dp_codec::A2DPCodec, bluetooth_protocol::BluetoothProtocol, bus::Bus,
    device_state::DeviceState, form_factor::FormFactor,
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardDevice {
    pub index: u64,
    pub name: String,
    pub driver: String,
    pub state: DeviceState,
    pub is_default: bool,
    pub volume: f64,
    pub is_muted: bool,
    pub card_index: u64,
    pub description: String,
    pub bluetooth_protocol: Option<BluetoothProtocol>,
    pub a2dp_codec: Option<A2DPCodec>,
    pub form_factor: FormFactor,
    pub bus: Bus,
}

impl Default for CardDevice {
    fn default() -> Self {
        CardDevice {
            index: default(),
            name: default(),
            driver: default(),
            state: default(),
            is_default: default(),
            volume: default(),
            is_muted: default(),
            card_index: default(),
            description: default(),
            bluetooth_protocol: default(),
            a2dp_codec: default(),
            form_factor: default(),
            bus: default(),
        }
    }
}
