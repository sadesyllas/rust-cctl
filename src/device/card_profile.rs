use string_enum_string::string_enum_string;

use serde_repr::{Deserialize_repr, Serialize_repr};

#[string_enum_string]
#[derive(Clone, Debug, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum CardProfile {
    #[variant((display = "Headset Head Unit (HSP/HFP)", parse = "headset_head_unit"))]
    HeadsetHeadUnit = 1,

    #[variant((display = "High Fidelity Playback (A2DP Sink: SBC)", parse = "a2dp_sink_sbc"))]
    A2DPSinkSBC = 2,

    #[variant((display = "High Fidelity Playback (A2DP Sink: AAC)", parse = "a2dp_sink_aac"))]
    A2DPSinkAAC = 3,

    #[variant((display = "High Fidelity Playback (A2DP Sink: aptX)", parse = "a2dp_sink_aptx"))]
    A2DPSinkAptX = 4,

    #[variant((display = "High Fidelity Playback (A2DP Sink: aptX HD)", parse = "a2dp_sink_aptx_hd"))]
    A2DPSinkAptXHD = 5,

    #[variant((display = "High Fidelity Playback (A2DP Sink: LDAC)", parse = "a2dp_sink_ldac"))]
    A2DPSinkLDAC = 6,

    Off = 7,
}

impl Default for CardProfile {
    fn default() -> Self {
        Self::Off
    }
}
