use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub enum CardDeviceType {
    #[serde(alias = "source")]
    Source,

    #[serde(alias = "sink")]
    Sink,
}

impl std::fmt::Display for CardDeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CardDeviceType::Source => f.write_str("Source").unwrap(),
            CardDeviceType::Sink => f.write_str("Sink").unwrap(),
        }

        Ok(())
    }
}
