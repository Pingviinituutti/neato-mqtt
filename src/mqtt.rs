use eyre::Result;
use rand::{distributions::Alphanumeric, Rng};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::{sync::watch::Receiver, task};

use crate::settings::MqttSettings;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct MqttDevice {
    pub id: String,
    pub name: Option<String>,
    pub power: Option<bool>,
    pub volume: Option<u16>,
    pub mute: Option<bool>,
}

#[derive(Clone)]
pub struct MqttClient {
    pub client: AsyncClient,
    pub rx: Receiver<Option<MqttDevice>>,
    pub topic: String,
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
    client
        .subscribe(mqtt_settings.topic_set.clone(), QoS::AtMostOnce)
        .await?;

    let (tx, rx) = tokio::sync::watch::channel(None);

    task::spawn(async move {
        loop {
            let notification = eventloop.poll().await;

            let res = (|| async {
                if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg)) = notification? {
                    let device: MqttDevice = serde_json::from_slice(&msg.payload)?;
                    tx.send(Some(device))?;
                }

                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            })()
            .await;

            if let Err(e) = res {
                eprintln!("MQTT error: {:?}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    });

    Ok(MqttClient {
        client,
        rx,
        topic: mqtt_settings.topic.clone(),
    })
}
