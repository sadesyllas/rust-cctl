use serde_repr::Serialize_repr;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Serialize_repr)]
#[repr(u8)]
pub enum A2DPCodec {
    SBC = 1,
    AAC = 2,
    AptX = 3,
    AptXHD = 4,
    LDAC = 5,
}
