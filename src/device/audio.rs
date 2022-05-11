use std::process::Stdio;

use futures::future::join_all;
use log::{error, info};
use tokio::{
    io::{self, AsyncReadExt},
    process::Command,
};
use tracing::instrument;

use crate::device::{
    audio_client, parse_card_devices::parse_card_devices, parse_cards::parse_cards,
};

use super::{
    card::Card, card_device::CardDevice, card_device_type::CardDeviceType,
    card_profile::CardProfile,
};

#[instrument]
pub async fn fetch_devices() -> (Vec<Card>, Vec<CardDevice>, Vec<CardDevice>) {
    let cards = fetch_cards();
    let sources = fetch_card_devices(CardDeviceType::Source);
    let sinks = fetch_card_devices(CardDeviceType::Sink);

    tokio::try_join!(cards, sources, sinks).unwrap()
}

#[instrument]
pub async fn set_volume(
    _type: CardDeviceType,
    index: u64,
    volume_percentage: f64,
) -> io::Result<()> {
    let subcommand = match _type {
        CardDeviceType::Source => "set-source-volume",
        CardDeviceType::Sink => "set-sink-volume",
    };

    let volume_percentage: f64 = volume_percentage.min(100.0);
    let volume = ((((volume_percentage * 65535.0 / 100.0) * 10.0).round() / 10.0).round() as u64)
        .to_string();

    let mut command = Command::new("pacmd")
        .args(&[subcommand, index.to_string().as_str(), &volume])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let exit_status = command.wait().await?;

    if !exit_status.success() {
        error!(
            "Could not set the volume of {} index {} to {}%",
            _type, index, volume_percentage
        );
    }

    Ok(())
}

#[instrument]
pub async fn toggle_mute(_type: CardDeviceType, index: u64, mute: bool) -> io::Result<()> {
    let subcommand = match _type {
        CardDeviceType::Source => "set-source-mute",
        CardDeviceType::Sink => "set-sink-mute",
    };

    let mute_value = if mute { "true" } else { "false" };

    let mut command = Command::new("pacmd")
        .args(&[subcommand, index.to_string().as_str(), mute_value])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let exit_status = command.wait().await?;

    if !exit_status.success() {
        error!(
            "Could not set mute of {} index {} to mute status {}",
            _type, index, mute_value
        );
    }

    Ok(())
}

#[instrument]
pub async fn set_default_card_device(_type: CardDeviceType, index: u64) -> io::Result<()> {
    let subcommand = match _type {
        CardDeviceType::Source => "set-default-source",
        CardDeviceType::Sink => "set-default-sink",
    };

    let mut command = Command::new("pacmd")
        .args(&[subcommand, index.to_string().as_str()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let exit_status = command.wait().await?;

    if !exit_status.success() {
        error!(
            "Could not set the {} index {} as the default {}",
            _type, index, _type
        );
    }

    Ok(())
}

#[instrument]
pub async fn set_card_profile(index: u64, profile: CardProfile) -> io::Result<()> {
    let mut command = Command::new("pacmd")
        .args(&[
            "set-card-profile",
            index.to_string().as_str(),
            profile.to_pa_str().as_str(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let exit_status = command.wait().await?;

    if !exit_status.success() {
        error!(
            "Could not set the card index {} to profile {}",
            index,
            format!("{}", profile).as_str()
        );
    }

    Ok(())
}

#[instrument]
pub async fn move_audio_clients(_type: CardDeviceType, index: u64, name: &str) -> io::Result<()> {
    let clients = audio_client::fetch_client_indexes(_type).await?;
    let mut moves = Vec::new();

    for (client_index, current_index) in clients {
        if current_index != index {
            moves.push(async move {
                audio_client::set_client_card_device(client_index, _type, name).await?;

                info!(
                    "Moved audio client index {} to default {} {}",
                    client_index, _type, name
                );

                io::Result::Ok(())
            });
        }
    }

    join_all(moves).await;

    Ok(())
}

#[instrument]
async fn fetch_cards() -> io::Result<Vec<Card>> {
    let mut command = Command::new("pacmd")
        .args(&["list-cards"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let exit_status = command.wait().await?;

    if !exit_status.success() {
        error!("Could not get card information");
    }

    let mut output = String::new();

    if let Some(mut stdout) = command.stdout.take() {
        stdout.read_to_string(&mut output).await?;
    }

    Ok(parse_cards(&output))
}

#[instrument]
async fn fetch_card_devices(_type: CardDeviceType) -> io::Result<Vec<CardDevice>> {
    let argument = match _type {
        CardDeviceType::Source => "list-sources",
        CardDeviceType::Sink => "list-sinks",
    };

    let mut command = Command::new("pacmd")
        .args(&[argument])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let exit_status = command.wait().await?;

    if !exit_status.success() {
        error!("Could not get {} information", _type);
    }

    let mut output = String::new();

    if let Some(mut stdout) = command.stdout.take() {
        stdout.read_to_string(&mut output).await?;
    }

    Ok(parse_card_devices(&output))
}
