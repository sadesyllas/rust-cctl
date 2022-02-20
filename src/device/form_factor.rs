use std::fmt::Display;

use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum FormFactor {
    Internal = 1,
    Headphones = 2,
    Webcam = 3,
    Headset = 4,
}

impl FormFactor {
    pub fn from_pa_str(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "internal" => FormFactor::Internal,
            "headphone" => FormFactor::Headphones,
            "webcam" => FormFactor::Webcam,
            "headset" => FormFactor::Headset,
            _ => unreachable!(),
        }
    }
}

impl Display for FormFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormFactor::Internal => f.write_str("internal"),
            FormFactor::Headphones => f.write_str("headphones"),
            FormFactor::Webcam => f.write_str("webcam"),
            FormFactor::Headset => f.write_str("headset"),
        }
    }
}

impl Default for FormFactor {
    fn default() -> Self {
        FormFactor::Internal
    }
}
