use eyre::Result;
use rand::{distributions::Alphanumeric, Rng};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use std::{time::Duration, sync::Arc};
use tokio::{sync::watch::Receiver, task};

use log::{debug, error};

use crate::{settings::MqttSettings, neato::RobotCmd};

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

pub fn get_id_from_topic(topic: &String, topic_set: &String) -> Result<String> {
    if let Some((start, end)) = topic_set.split_once("{id}") {
        Ok(topic.replace(start, "").replace(end, ""))
    } else {
        error!("Could not get id from topic: '{}'", topic);
        Err(eyre::eyre!("Could not get id from topic: '{}'", topic))
    }
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

    task::spawn(async move {
        loop {
            let notification = eventloop.poll().await;

            let id = subscribe_settings.id.clone();
            let config_clone = Arc::clone(&config_clone);

            let res = (|| async {
                match notification? {
                    rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_)) => {
                        subscribe_client
                            .subscribe(config_clone.topic_set.replace("{id}", "+"), QoS::AtMostOnce)
                            .await?;
                    }
                    rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg)) => {
                        debug!("Reveiced MQTT Publish for topic: {:?}", &msg.topic);
                        let id = get_id_from_topic(&msg.topic, &config_clone.topic_set)?;
                        debug!("Id is: {:?}", id);
                        debug!("Payload is: {:?}", &msg.payload);
                        let payload: MqttSetMessage = serde_json::from_slice(&msg.payload)?;
                        let device  = {SendAction {
                            id: id,
                            action: payload.action,
                        }};
                        tx.send(Some(device))?;
                    }
                    _ => {}
                }

                Ok::<(), Box<dyn std::error::Error + Sync + Send>>(())
            })()
            .await;

            if let Err(e) = res {
                error!(
                    target: &id.to_string(),
                    "MQTT error: {:?}", e
                );
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    });

    Ok(MqttClient {
        client,
        rx,
        topic: mqtt_settings.topic.clone(),
        set_topic: mqtt_settings.topic_set.clone(),
        settings: mqtt_settings.clone(),
    })
}
