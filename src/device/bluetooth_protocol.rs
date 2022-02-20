use std::fmt::Display;

use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum BluetoothProtocol {
    HeadsetHeadUnit = 1,
    A2DPSink = 2,
}

impl BluetoothProtocol {
    pub fn from_pa_str(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "headset_head_unit" => BluetoothProtocol::HeadsetHeadUnit,
            "a2dp_sink" => BluetoothProtocol::A2DPSink,
            _ => unreachable!(),
        }
    }
}

impl Display for BluetoothProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BluetoothProtocol::HeadsetHeadUnit => f.write_str("Headset Head Unit (HSP/HFP)"),
            BluetoothProtocol::A2DPSink => f.write_str("A2DP Sink"),
        }
    }
}
