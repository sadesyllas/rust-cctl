use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub enum CardDeviceType {
    #[serde(alias = "source")]
    Source,

    #[serde(alias = "sink")]
    Sink,
}
