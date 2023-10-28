use std::env;

use log::log_enabled;
use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct NeatoSettings {
    pub email: String,
    pub password: String,
    pub poll_interval: u16, // seconds
    pub cache_timeout: u16, // seconds
    pub decode_state: bool,
    pub dry_run: bool,
}

fn default_poll_interval() -> u16 {
    if log_enabled!(log::Level::Debug) {
        5 // seconds
    } else {
        60 // 1 minute
    }
}

fn default_cache_timeout() -> u16 {
    if log_enabled!(log::Level::Debug) {
        30 // seconds
    } else {
        5 * 60 // 5 minutes
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct MqttSettings {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub topic: String,
    pub topic_set: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Settings {
    pub neato: NeatoSettings,
    pub mqtt: MqttSettings,
}

pub fn read_settings() -> Result<Settings, config::ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("Settings"))
        .set_default("mqtt.host", "localhost")?
        .set_default("mqtt.port", 1883)?
        .set_default("mqtt.topic", "home/devices/neato/{id}")?
        .set_default("mqtt.topic_set", "home/devices/neato/{id}/set")?
        .set_default("neato.poll_interval", default_poll_interval())?
        .set_default("neato.cache_timeout", default_cache_timeout())?
        .set_default("neato.decode_state", false)?
        .set_default("neato.dry_run", false)?
        .set_override_option("mqtt.host", env::var("MQTT_HOST").ok())?
        .build()?
        .try_deserialize::<Settings>()
}
