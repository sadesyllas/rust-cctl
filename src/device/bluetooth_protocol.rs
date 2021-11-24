use string_enum_string::string_enum_string;

use serde_repr::Serialize_repr;

#[derive(Clone, Debug, Serialize_repr)]
#[repr(u8)]
#[string_enum_string]
pub enum BluetoothProtocol {
    #[variant((display = "Headset Head Unit (HSP/HFP)", parse = "headset_head_unit"))]
    HeadsetHeadUnit = 1,

    #[variant((display = "A2DP Sink", parse = "a2dp_sink"))]
    A2DPSink = 2,
}
