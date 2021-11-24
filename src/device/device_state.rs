use string_enum_string::string_enum_string;

use serde_repr::Serialize_repr;

#[derive(Clone, Debug, Serialize_repr)]
#[repr(u8)]
#[string_enum_string]
pub enum DeviceState {
    #[variant(display = "running")]
    Running = 1,

    #[variant(display = "idle")]
    Idle = 2,

    #[variant(display = "suspended")]
    Suspended = 3,
}

impl Default for DeviceState {
    fn default() -> Self {
        DeviceState::Suspended
    }
}
