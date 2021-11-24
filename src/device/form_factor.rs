use string_enum_string::string_enum_string;

use serde_repr::Serialize_repr;

#[derive(Clone, Debug, Serialize_repr)]
#[repr(u8)]
#[string_enum_string]
pub enum FormFactor {
    #[variant(display = "internal")]
    Internal = 1,

    #[variant((display = "headphones", parse = "headphone"))]
    Headphones = 2,

    #[variant(display = "webcam")]
    Webcam = 3,

    #[variant(display = "headset", parse = "headset")]
    Headset = 4,
}

impl Default for FormFactor {
    fn default() -> Self {
        FormFactor::Internal
    }
}
