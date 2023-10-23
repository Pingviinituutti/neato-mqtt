use log::log_enabled;
use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct NeatoSettings {
    pub email: String,
    pub password: String,
    #[serde(default = "default_poll_intervall")]
    pub poll_intervall: u16, // seconds
    #[serde(default = "default_cache_timeout")]
    pub cache_timeout: u16, // seconds
    #[serde(default = "default_decode_state")]
    pub decode_state: bool,
}

fn default_decode_state() -> bool { false }

fn default_poll_intervall() -> u16 {
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
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_topic")]
    pub topic: String,
    #[serde(default = "default_set_topic")]
    pub topic_set: String,
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_port() -> u16 {
    1883
}

fn default_topic() -> String {
    "home/devices/neato/{id}".to_string()
}

fn default_set_topic() -> String {
    format!("{}/set", default_topic())
}

#[derive(Clone, Deserialize, Debug)]
pub struct Settings {
    pub neato: NeatoSettings,
    pub mqtt: MqttSettings,
}

pub fn read_settings() -> Result<Settings, config::ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("Settings"))
        .build()?
        .try_deserialize::<Settings>()
}
