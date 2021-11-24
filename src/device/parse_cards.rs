use std::default::default;

use regex::Regex;

use crate::{device::bus::Bus, util::unquote_parsed_string_value};

use super::card::Card;

struct ParseContext {
    in_profiles: bool,
    in_sources: bool,
    in_sinks: bool,
}

pub fn parse_cards(text: &str) -> Vec<Card> {
    let mut cards: Vec<Card> = Vec::new();
    let mut current_card: Option<Card> = None;
    let mut parse_context = ParseContext {
        in_profiles: false,
        in_sources: false,
        in_sinks: false,
    };

    text.lines().map(|line| line.trim()).for_each(|line| {
        let captures = Regex::new(r"(?P<key>(?:\*\s*)?[^:=]+?)\s*[:=]\s*(?P<value>.+$)?")
            .unwrap()
            .captures(line);

        if let Some(captures) = captures {
            match captures.name("key").unwrap().as_str() {
                "index" => {
                    if current_card.is_some() {
                        cards.push(current_card.take().unwrap());
                    }

                    let mut current: Card = default();
                    let value = captures.name("value").unwrap().as_str();

                    current.index = value.parse().unwrap();

                    current_card.replace(current);
                }
                _match @ ("name" | "driver" | "device.description") => {
                    let mut current: Card = current_card.take().unwrap();
                    let value = captures.name("value").unwrap().as_str();
                    let value = unquote_parsed_string_value(value);

                    match _match {
                        "name" => current.name = value,
                        "driver" => current.driver = value,
                        "device.description" => current.description = value,
                        _ => unreachable!(),
                    }

                    current_card.replace(current);
                }
                "profiles" => parse_context.in_profiles = true,
                "active profile" => {
                    parse_context.in_profiles = false;

                    let mut current: Card = current_card.take().unwrap();

                    if current.bus == Bus::Bluetooth {
                        let value = captures.name("value").unwrap().as_str();
                        let value = unquote_parsed_string_value(value);

                        current.active_profile = value.as_str().into();
                    }

                    current_card.replace(current);
                }
                "sinks" => parse_context.in_sinks = true,
                "sources" => {
                    parse_context.in_sinks = false;
                    parse_context.in_sources = true;
                }
                "ports" => parse_context.in_sources = false,
                "device.form_factor" => {
                    let mut current: Card = current_card.take().unwrap();
                    let value = captures.name("value").unwrap().as_str();
                    let value = unquote_parsed_string_value(value);

                    current.form_factor = value.as_str().into();

                    current_card.replace(current);
                }
                "device.bus" => {
                    let mut current: Card = current_card.take().unwrap();
                    let value = captures.name("value").unwrap().as_str();
                    let value = unquote_parsed_string_value(value);

                    current.bus = value.as_str().into();

                    current_card.replace(current);
                }
                matched => {
                    let mut current: Card = current_card.take().unwrap();

                    if parse_context.in_profiles && current.bus == Bus::Bluetooth {
                        current.profiles.push(matched.into());
                    }

                    if parse_context.in_sinks {
                        current
                            .sink_ids
                            .push(matched.split('#').nth(1).unwrap().parse().unwrap());
                    } else if parse_context.in_sources {
                        current
                            .source_ids
                            .push(matched.split('#').nth(1).unwrap().parse().unwrap());
                    }

                    current_card.replace(current);
                }
            }
        }
    });

    if current_card.is_some() {
        cards.push(current_card.take().unwrap());
    }

    cards
}
