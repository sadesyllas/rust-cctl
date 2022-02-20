use std::fmt::Display;

use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Debug, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum CardProfile {
    HeadsetHeadUnit = 1,
    A2DPSinkSBC = 2,
    A2DPSinkAAC = 3,
    A2DPSinkAptX = 4,
    A2DPSinkAptXHD = 5,
    A2DPSinkLDAC = 6,
    Off = 7,
}

impl CardProfile {
    pub fn from_pa_str(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "headset_head_unit" => CardProfile::HeadsetHeadUnit,
            "a2dp_sink_sbc" => CardProfile::A2DPSinkSBC,
            "a2dp_sink_aac" => CardProfile::A2DPSinkAAC,
            "a2dp_sink_aptx" => CardProfile::A2DPSinkAptX,
            "a2dp_sink_aptx_hd" => CardProfile::A2DPSinkAptXHD,
            "a2dp_sink_ldac" => CardProfile::A2DPSinkLDAC,
            _ => unreachable!(),
        }
    }
}

impl Display for CardProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CardProfile::HeadsetHeadUnit => f.write_str("Headset Head Unit (HSP/HFP)"),
            CardProfile::A2DPSinkSBC => f.write_str("High Fidelity Playback (A2DP Sink: SBC"),
            CardProfile::A2DPSinkAAC => f.write_str("High Fidelity Playback (A2DP Sink: AAC"),
            CardProfile::A2DPSinkAptX => f.write_str("High Fidelity Playback (A2DP Sink: AptX"),
            CardProfile::A2DPSinkAptXHD => f.write_str("High Fidelity Playback (A2DP Sink: AptXHD"),
            CardProfile::A2DPSinkLDAC => f.write_str("High Fidelity Playback (A2DP Sink: LDAC"),
            CardProfile::Off => f.write_str("Off"),
        }
    }
}

impl Default for CardProfile {
    fn default() -> Self {
        Self::Off
    }
}
