use std::fmt::Display;

use serde_repr::{Deserialize_repr, Serialize_repr};

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Bus {
    PCI = 1,
    Bluetooth = 2,
    USB = 3,
}

impl Bus {
    pub fn from_pa_str(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "pci" => Bus::PCI,
            "bluetooth" => Bus::Bluetooth,
            "usb" => Bus::USB,
            _ => unreachable!(),
        }
    }
}

impl Display for Bus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bus::PCI => f.write_str("PCI"),
            Bus::Bluetooth => f.write_str("Bluetooth"),
            Bus::USB => f.write_str("USB"),
        }
    }
}

impl Default for Bus {
    fn default() -> Self {
        Bus::PCI
    }
}
