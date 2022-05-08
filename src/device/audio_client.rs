use std::{default::default, process::Stdio};

use regex::Regex;
use tokio::{
    io::{self, AsyncReadExt},
    process::Command,
};
use tracing::{error, instrument};

use super::card_device_type::CardDeviceType;

#[instrument]
pub async fn fetch_client_indexes(_type: CardDeviceType) -> io::Result<Vec<(u64, u64)>> {
    let arg = if _type == CardDeviceType::Source {
        "list-source-outputs"
    } else {
        "list-sink-inputs"
    };

    let mut command = Command::new("pacmd")
        .args(&[arg])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let exit_status = command.wait().await?;

    if !exit_status.success() {
        error!("Could not get {} client ids", _type);

        return Ok(default());
    }

    let mut output = String::new();

    if let Some(mut stdout) = command.stdout.take() {
        stdout.read_to_string(&mut output).await?;
    }

    Ok(parse_client_indexes(&output))
}

#[instrument]
pub async fn set_client_card_device(
    client_index: u64,
    _type: CardDeviceType,
    card_device_name: &str,
) -> io::Result<()> {
    let arg = if _type == CardDeviceType::Source {
        "move-source-output"
    } else {
        "move-sink-input"
    };

    let mut command = Command::new("pacmd")
        .args(&[arg, client_index.to_string().as_str(), card_device_name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let exit_status = command.wait().await?;

    if !exit_status.success() {
        error!(
            "Could not set client index {} to {} {}",
            client_index, _type, card_device_name
        );
    }

    Ok(())
}

fn parse_client_indexes(text: &str) -> Vec<(u64, u64)> {
    let mut indexes: Vec<(u64, u64)> = Vec::new();
    let mut current_index: u64 = 0;
    let mut current_source_sink_index: u64 = 0;
    let mut is_monitor = false;

    text.lines().map(|line| line.trim()).for_each(|line| {
        let captures = Regex::new(r"(?P<key>(?:\*\s*)?[^:=]+?)\s*[:=]\s*(?P<value>.+$)")
            .unwrap()
            .captures(line);

        if let Some(captures) = captures {
            match captures.name("key").unwrap().as_str() {
                "index" => {
                    current_index = captures.name("value").unwrap().as_str().parse().unwrap();
                }
                "source" | "sink" => {
                    let source_sink_captures =
                        Regex::new(r"\s*(?P<index>[0-9]+)\s*<(?P<name>[^>]+)>")
                            .unwrap()
                            .captures(captures.name("value").unwrap().as_str())
                            .unwrap();

                    current_source_sink_index = source_sink_captures
                        .name("index")
                        .unwrap()
                        .as_str()
                        .parse()
                        .unwrap();

                    let source_sink_name: &str =
                        source_sink_captures.name("name").unwrap().as_str();

                    is_monitor = source_sink_name.ends_with(".monitor");
                }
                "client" => {
                    if is_monitor {
                        return;
                    }

                    let client_captures = Regex::new(r"\s*(?:[0-9]+)\s*<(?P<name>[^>]+)>")
                        .unwrap()
                        .captures(captures.name("value").unwrap().as_str())
                        .unwrap();

                    if client_captures.name("name").unwrap().as_str() == "PulseAudio Volume Control"
                    {
                        return;
                    }

                    indexes.push((current_index, current_source_sink_index));
                }
                _ => (),
            }
        }
    });

    indexes
}
