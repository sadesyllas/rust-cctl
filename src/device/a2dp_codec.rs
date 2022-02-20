use std::fmt::Display;

use serde_repr::{Deserialize_repr, Serialize_repr};

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum A2DPCodec {
    SBC = 1,
    AAC = 2,
    AptX = 3,
    AptXHD = 4,
    LDAC = 5,
}

impl A2DPCodec {
    pub fn from_pa_str(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "sbc" => A2DPCodec::SBC,
            "aac" => A2DPCodec::AAC,
            "aptx" => A2DPCodec::AptX,
            "aptx_hd" => A2DPCodec::AptXHD,
            "ldac" => A2DPCodec::LDAC,
            _ => unreachable!(),
        }
    }
}

impl Display for A2DPCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            A2DPCodec::SBC => f.write_str("SBC"),
            A2DPCodec::AAC => f.write_str("AAC"),
            A2DPCodec::AptX => f.write_str("AptX"),
            A2DPCodec::AptXHD => f.write_str("AptXHD"),
            A2DPCodec::LDAC => f.write_str("LDAC"),
        }
    }
}
