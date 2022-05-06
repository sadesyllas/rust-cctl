use serde::Serialize;

use super::{bus::Bus, card_profile::CardProfile, form_factor::FormFactor};

#[derive(Clone, Debug, Default, Serialize)]
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
