extern crate pretty_env_logger;
extern crate log;

mod mqtt;
mod neato;
mod neato_types;
mod settings;

use color_eyre::Result;
use neato::Neato;

use crate::settings::read_settings;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    pretty_env_logger::init();

    let settings = read_settings()?;
    // let mqtt_client = mk_mqtt_client(&settings).await?;
    let mqtt_client = mqtt::init(&settings.mqtt.clone()).await?;
    let _neato = Neato::new(mqtt_client, &settings.neato.clone())
        .init().await?;

    tokio::signal::ctrl_c().await?;

    Ok(())
}
