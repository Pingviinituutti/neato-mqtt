use eyre::Result;
use rand::{distributions::Alphanumeric, Rng};
use rumqttc::{AsyncClient, ConnectionError, Event, MqttOptions, Publish, QoS};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::{sync::watch::Receiver, task};

use log::{debug, error};

use crate::{neato::RobotCmd, settings::MqttSettings};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct MqttSetMessage {
    pub action: RobotCmd,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct SendAction {
    pub id: String,
    pub action: RobotCmd,
}

#[derive(Clone)]
pub struct MqttClient {
    pub client: AsyncClient,
    pub rx: Receiver<Option<SendAction>>,
    pub topic: String,
    pub set_topic: String,
    pub settings: MqttSettings,
}

pub fn get_id_from_topic(topic: &String, set_topic: &str) -> Result<String> {
    if let Some((start, end)) = set_topic.split_once("{id}") {
        Ok(topic.replace(start, "").replace(end, ""))
    } else {
        error!("Could not get id from topic: '{}'", topic);
        Err(eyre::eyre!("Could not get id from topic: '{}'", topic))
    }
}

struct NotificationResult {
    message: Option<Publish>,
}

async fn handle_notification(
    client: &AsyncClient,
    notification: Result<Event, ConnectionError>,
    mqtt_settings: &MqttSettings,
) -> Result<NotificationResult> {
    debug!("Notification: {:?}", notification);
    match notification? {
        rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) => {
            client
                .subscribe(mqtt_settings.get_set_topic_with_wildcard(), QoS::AtMostOnce)
                .await?;
            client
                .subscribe(mqtt_settings.get_topic_with_id_as_set(), QoS::AtMostOnce)
                .await?;
            // return Ok(NotificationResult{message: None})
        }
        rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg)) => {
            return Ok(NotificationResult { message: Some(msg) })
        }
        _ => {}
    }

    Ok(NotificationResult { message: None })
    // Err(eyre::eyre!("Could not get message"))
}

pub async fn init(mqtt_settings: &MqttSettings) -> Result<MqttClient> {
    let random_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let mut options = MqttOptions::new(
        format!("{}-{}", mqtt_settings.id.clone(), random_string),
        mqtt_settings.host.clone(),
        mqtt_settings.port,
    );
    options.set_keep_alive(Duration::from_secs(5));
    let (client, mut eventloop) = AsyncClient::new(options, 10);

    let subscribe_settings = mqtt_settings.clone();
    let subscribe_client = client.clone();

    let config_clone = Arc::new(mqtt_settings.clone());

    let (tx, rx) = tokio::sync::watch::channel(None);

    // Listen on set_topic, for example `home/devices/neato/{id}/set`
    task::spawn(async move {
        loop {
            let notification = eventloop.poll().await;

            let id = subscribe_settings.id.clone();
            let config_clone = Arc::clone(&config_clone);

            let res =
                handle_notification(&subscribe_client, notification, &subscribe_settings.clone())
                    .await;

            match res {
                Ok(NotificationResult { message: Some(msg) }) => {
                    debug!("Reveiced MQTT Publish for topic: {:?}", &msg.topic);
                    let id = get_id_from_topic(&msg.topic, &config_clone.set_topic).unwrap();
                    debug!("Id is: {:?}", id);
                    let payload: MqttSetMessage = match serde_json::from_slice(&msg.payload) {
                        Ok(pl) => pl,
                        Err(e) => {
                            error!("Could not parse JSON payload: {:?}", e);
                            continue;
                        }
                    };
                    debug!("Payload is: {:?}", payload);
                    let device = {
                        SendAction {
                            id,
                            action: payload.action,
                        }
                    };
                    tx.send(Some(device)).expect("Failed to send message");
                }
                Err(e) => {
                    error!(
                        target: &id.to_string(),
                        "MQTT error: {:?}", e
                    );
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                _ => {}
            }
        }
    });

    // listen on the topic set, for example `home/devices/neato/set`
    // If an action is received here, the command should affect all devices
    // TODO: implement this

    Ok(MqttClient {
        client,
        rx,
        topic: mqtt_settings.topic.clone(),
        set_topic: mqtt_settings.set_topic.clone(),
        settings: mqtt_settings.clone(),
    })
}
