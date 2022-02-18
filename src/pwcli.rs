use std::{fmt::Display, process::Stdio};

use serde::Serialize;
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use tokio::process::Command;
use tracing::error;

// use crate::pubsub::message::MessagePayload;

#[derive(Clone, Copy, Debug, Serialize_repr)]
#[repr(u8)]
enum ObjectType {
    Device = 1,
    Source = 2,
    Sink = 3,
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ObjectType::Device => f.write_str("Device"),
            ObjectType::Source => f.write_str("Source"),
            ObjectType::Sink => f.write_str("Sink"),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct AudioProfile {
    index: usize,
    name: String,
}

#[derive(Clone, Debug, Serialize)]
struct DeviceRoute {
    index: usize,
    direction: DeviceRouteDirection,
    mute: bool,
    volume: f64,
}

#[derive(Clone, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum DeviceRouteDirection {
    Input = 1,
    Output = 2,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    id: usize,
    r#type: ObjectType,
    default: bool,
    name: String,
    description: String,
    device_id: Option<usize>,
    bus: String,
    form_factor: String,
    profiles: Vec<AudioProfile>,
    profile: Option<AudioProfile>,
    routes: Vec<DeviceRoute>,
}

// impl MessagePayload for Vec<Object> {
//     fn as_any(&self) -> &dyn std::any::Any {
//         self
//     }
// }

pub async fn get_objects() -> Result<Vec<Object>, String> {
    let command = Command::new("pw-dump")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let result = command
        .wait_with_output()
        .await
        .map_err(|err| err.to_string())?;

    if !result.status.success() {
        error!(
            "Could not run pw-cli: {}",
            String::from_utf8(result.stderr.clone()).unwrap()
        );
    }

    let objects_text = String::from_utf8(result.stdout).map_err(|err| err.to_string())?;
    let mut parsed_objects: Vec<Object> = Vec::new();
    let objects: Value = serde_json::from_str(&objects_text).unwrap();
    let objects = objects.as_array().unwrap();
    let mut default_source_name: Option<String> = None;
    let mut default_sink_name: Option<String> = None;

    for object in objects {
        if object["type"] != Value::String("PipeWire:Interface:Metadata".to_string()) {
            continue;
        }

        for meta in object["metadata"].as_array().unwrap() {
            match meta["key"].as_str().unwrap() {
                "default.configured.audio.source" => {
                    default_source_name = Some(meta["value"]["name"].to_string())
                }
                "default.configured.audio.sink" => {
                    default_sink_name = Some(meta["value"]["name"].to_string())
                }
                _ => continue,
            }
        }
    }

    for object in objects {
        let valid_object_type = match object["info"]["props"]["media.class"]
            .as_str()
            .unwrap_or_default()
        {
            "Audio/Device" => Some(ObjectType::Device),
            "Audio/Source" => Some(ObjectType::Source),
            "Audio/Sink" => Some(ObjectType::Sink),
            _ => None,
        };

        if valid_object_type.is_none() {
            continue;
        }

        let props_prefix = match valid_object_type.unwrap() {
            ObjectType::Device => "device.".to_string(),
            ObjectType::Source => "node.".to_string(),
            ObjectType::Sink => "node.".to_string(),
        };

        let profiles = match &object["info"]["params"]["EnumProfile"] {
            Value::Null => None,
            Value::Array(values) => {
                let mut profiles: Vec<AudioProfile> = Vec::new();

                for value in values {
                    profiles.push(AudioProfile {
                        index: value["index"].as_u64().unwrap() as usize,
                        name: value["name"].to_string(),
                    });
                }

                Some(profiles)
            }
            _ => unreachable!(),
        };

        let profile = match &object["info"]["params"]["Profile"] {
            Value::Null => None,
            Value::Array(value) => Some(AudioProfile {
                index: value[0]["index"].as_u64().unwrap() as usize,
                name: value[0]["name"].to_string(),
            }),
            _ => unreachable!(),
        };

        let routes: Vec<DeviceRoute> = {
            if let Some(ObjectType::Device) = valid_object_type {
                let mut routes = Vec::new();

                for route in object["info"]["params"]["Route"].as_array().unwrap() {
                    routes.push(DeviceRoute {
                        index: route["index"].as_u64().unwrap() as usize,
                        direction: match route["direction"].as_str().unwrap() {
                            "Input" => DeviceRouteDirection::Input,
                            "Output" => DeviceRouteDirection::Output,
                            _ => unreachable!(),
                        },
                        mute: route["props"]["mute"].as_bool().unwrap(),
                        volume: route["props"]["channelVolumes"][0].as_f64().unwrap(),
                    })
                }

                routes
            } else {
                Vec::new()
            }
        };

        let name = object["info"]["props"][props_prefix.clone() + "name"].to_string();

        let default = name
            == default_source_name
                .as_ref()
                .unwrap_or(&"".to_string())
                .as_str()
            || name
                == default_sink_name
                    .as_ref()
                    .unwrap_or(&"".to_string())
                    .as_str();

        parsed_objects.push(Object {
            id: object["id"].as_u64().unwrap() as usize,
            r#type: valid_object_type.unwrap(),
            default,
            name,
            description: object["info"]["props"][props_prefix.clone() + "description"].to_string(),
            bus: object["info"]["props"][props_prefix.clone() + "bus"].to_string(),
            form_factor: object["info"]["props"][props_prefix.clone() + "form-factor"].to_string(),
            device_id: match &object["info"]["props"]["device.id"] {
                Value::Null => None,
                Value::Number(value) => Some(value.as_u64().unwrap() as usize),
                _ => unreachable!(),
            },
            profiles: profiles.unwrap_or_default(),
            profile,
            routes,
        });
    }

    Ok(parsed_objects)
}
