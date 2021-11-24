use string_enum_string::string_enum_string;

use serde_repr::Serialize_repr;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, PartialEq, Serialize_repr)]
#[repr(u8)]
#[string_enum_string]
pub enum Bus {
    PCI = 1,
    Bluetooth = 2,
    USB = 3,
}

impl Default for Bus {
    fn default() -> Self {
        Bus::PCI
    }
}
