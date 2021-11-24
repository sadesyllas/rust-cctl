use string_enum_string::string_enum_string;

use serde_repr::Serialize_repr;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Serialize_repr)]
#[string_enum_string]
#[repr(u8)]
pub enum A2DPCodec {
    SBC = 1,
    AAC = 2,
    AptX = 3,

    #[variant(display = "AptX HD", parse = "aptX_HD")]
    AptXHD = 4,

    LDAC = 5,
}
