use std::default::default;

use regex::Regex;

use crate::util::unquote_parsed_string_value;

use super::{
    a2dp_codec::A2DPCodec, bluetooth_protocol::BluetoothProtocol, bus::Bus,
    card_device::CardDevice, device_state::DeviceState, form_factor::FormFactor,
};

pub fn parse_card_devices(text: &str) -> Vec<CardDevice> {
    let mut card_devices: Vec<CardDevice> = Vec::new();
    let mut current_card_device: Option<CardDevice> = None;

    text.lines().map(|line| line.trim()).for_each(|line| {
        let captures = Regex::new(r"(?P<key>(?:\*\s*)?[^:=]+?)\s*[:=]\s*(?P<value>.+$)")
            .unwrap()
            .captures(line);

        if let Some(captures) = captures {
            match captures.name("key").unwrap().as_str() {
                _match @ ("* index" | "index") => {
                    if current_card_device.is_some() {
                        card_devices.push(current_card_device.take().unwrap());
                    }

                    let mut current: CardDevice = default();
                    let value = captures.name("value").unwrap().as_str();

                    handle_index(_match, value, &mut current);

                    current_card_device.replace(current);
                }
                _match @ ("name" | "driver" | "device.description") => {
                    if current_card_device.is_none() {
                        return;
                    }

                    let mut current: CardDevice = current_card_device.take().unwrap();
                    let value = captures.name("value").unwrap().as_str();
                    let value = unquote_parsed_string_value(value);

                    match _match {
                        "name" => current.name = value,
                        "driver" => current.driver = value,
                        "device.description" => current.description = value,
                        _ => unreachable!(),
                    }

                    current_card_device.replace(current);
                }
                "muted" => {
                    if current_card_device.is_none() {
                        return;
                    }

                    let mut current: CardDevice = current_card_device.take().unwrap();

                    current.is_muted = captures.name("value").unwrap().as_str() == "yes";

                    current_card_device.replace(current);
                }
                "card" => {
                    if current_card_device.is_none() {
                        return;
                    }

                    let mut current: CardDevice = current_card_device.take().unwrap();

                    current.card_index = captures
                        .name("value")
                        .unwrap()
                        .as_str()
                        .split(' ')
                        .next()
                        .unwrap()
                        .parse()
                        .unwrap();

                    current_card_device.replace(current);
                }
                "device.form_factor" => {
                    if current_card_device.is_none() {
                        return;
                    }

                    let mut current: CardDevice = current_card_device.take().unwrap();
                    let value = captures.name("value").unwrap().as_str();
                    let value = unquote_parsed_string_value(value);

                    current.form_factor = FormFactor::from_pa_str(value.as_str());

                    current_card_device.replace(current);
                }
                "state" => {
                    if current_card_device.is_none() {
                        return;
                    }

                    let mut current: CardDevice = current_card_device.take().unwrap();

                    current.state =
                        DeviceState::from_pa_str(captures.name("value").unwrap().as_str());

                    current_card_device.replace(current);
                }
                "volume" => {
                    if current_card_device.is_none() {
                        return;
                    }

                    let mut current: CardDevice = current_card_device.take().unwrap();
                    let value = captures.name("value").unwrap().as_str();

                    handle_volume(value, &mut current);

                    current_card_device.replace(current);
                }
                "bluetooth.protocol" => {
                    if current_card_device.is_none() {
                        return;
                    }

                    let mut current: CardDevice = current_card_device.take().unwrap();
                    let value = captures.name("value").unwrap().as_str();
                    let value = unquote_parsed_string_value(value);

                    current.bluetooth_protocol =
                        Some(BluetoothProtocol::from_pa_str(value.as_str()));

                    current_card_device.replace(current);
                }
                "bluetooth.a2dp_codec" => {
                    if current_card_device.is_none() {
                        return;
                    }

                    let mut current: CardDevice = current_card_device.take().unwrap();
                    let value = captures.name("value").unwrap().as_str();
                    let value = unquote_parsed_string_value(value);

                    current.a2dp_codec = Some(A2DPCodec::from_pa_str(value.as_str()));

                    current_card_device.replace(current);
                }
                "device.bus" => {
                    if current_card_device.is_none() {
                        return;
                    }

                    let mut current: CardDevice = current_card_device.take().unwrap();
                    let value = captures.name("value").unwrap().as_str();
                    let value = unquote_parsed_string_value(value);

                    current.bus = Bus::from_pa_str(value.as_str());

                    current_card_device.replace(current);
                }
                "monitor_of" => {
                    current_card_device.take();
                }
                _ => (),
            }
        }
    });

    if current_card_device.is_some() {
        card_devices.push(current_card_device.take().unwrap());
    }

    card_devices
}

fn handle_index(key: &str, value: &str, current_source_sink: &mut CardDevice) {
    if key.starts_with('*') {
        current_source_sink.is_default = true;
    }

    current_source_sink.index = value.parse().unwrap();
}

fn handle_volume(value: &str, current_source_sink: &mut CardDevice) {
    let volume: f64 = Regex::new(r"^[^:]+:\s*(?P<volume>[0-9]+).*")
        .unwrap()
        .captures(value)
        .unwrap()
        .name("volume")
        .unwrap()
        .as_str()
        .parse()
        .unwrap();

    current_source_sink.volume = ((volume / 65535.0) * 100.0).round()
}
