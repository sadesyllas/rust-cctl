use std::fmt::Display;

use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum DeviceState {
    Running = 1,
    Idle = 2,
    Suspended = 3,
}

impl DeviceState {
    pub fn from_pa_str(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "running" => DeviceState::Running,
            "idle" => DeviceState::Idle,
            "suspended" => DeviceState::Suspended,
            _ => unreachable!(),
        }
    }
}

impl Display for DeviceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceState::Running => f.write_str("running"),
            DeviceState::Idle => f.write_str("idle"),
            DeviceState::Suspended => f.write_str("suspended"),
        }
    }
}

impl Default for DeviceState {
    fn default() -> Self {
        DeviceState::Suspended
    }
}
