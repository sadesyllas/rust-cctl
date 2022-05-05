use std::default::default;

use serde::Serialize;

use super::{bus::Bus, card_profile::CardProfile, form_factor::FormFactor};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub index: u64,
    pub name: String,
    pub driver: String,
    pub description: String,
    pub profiles: Vec<CardProfile>,
    pub active_profile: CardProfile,
    pub source_ids: Vec<u64>,
    pub sink_ids: Vec<u64>,
    pub form_factor: FormFactor,
    pub bus: Bus,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            index: default(),
            name: default(),
            driver: default(),
            description: default(),
            profiles: Vec::new(),
            active_profile: default(),
            source_ids: Vec::new(),
            sink_ids: Vec::new(),
            form_factor: default(),
            bus: default(),
        }
    }
}
