use std::{
    fmt,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::sync::Mutex as AsyncMutex;

use chrono::Utc;
use color_eyre::Result;
use eyre::eyre;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use log::{debug, error, info};

use crate::{neato_types::{HouseCleaningParams, NeatoState, PublicRobot, Robot, RobotMessage}, mqtt::SendAction};
use crate::{mqtt::MqttClient, settings::NeatoSettings};

impl Robot {
    pub async fn publish(&self, mqtt_client: MqttClient) -> color_eyre::Result<()> {
        let topic = mqtt_client
            .topic
            .clone()
            .replace("{id}", self.name.as_str());

        let v = serde_json::to_value(self).unwrap();
        let public_robot: PublicRobot = serde_json::from_value(v).unwrap();
        mqtt_client
            .client
            .publish(
                topic,
                rumqttc::QoS::AtMostOnce,
                false,
                serde_json::to_string(&public_robot)?,
            )
            .await
            .unwrap();

        Ok(())
    }
}

#[derive(Deserialize)]
struct SessionsResponse {
    access_token: String,
}

#[derive(Serialize)]
struct AuthBody {
    email: String,
    password: String,
}

const BASE_URL: &str = "https://beehive.neatocloud.com";

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
pub enum RobotCmd {
    StartCleaning,
    StopCleaning,
    PauseCleaning,
    ResumeCleaning,
    SendToBase,
    GetRobotState,
}

impl fmt::Display for RobotCmd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RobotCmd::StartCleaning => write!(f, "startCleaning"),
            RobotCmd::StopCleaning => write!(f, "stopCleaning"),
            RobotCmd::PauseCleaning => write!(f, "pauseCleaning"),
            RobotCmd::ResumeCleaning => write!(f, "resumeCleaning"),
            RobotCmd::SendToBase => write!(f, "sendToBase"),
            RobotCmd::GetRobotState => write!(f, "getRobotState"),
        }
    }
}

impl RobotCmd {
    pub fn build_robot_message(&self) -> RobotMessage {
        match self {
            RobotCmd::StartCleaning => RobotMessage {
                req_id: String::from("77"),
                cmd: String::from("startCleaning"),
                params: Some(HouseCleaningParams {
                    category: 4,
                    mode: 1,
                    navigation_mode: 2,
                }),
            },
            other => RobotMessage {
                req_id: String::from("77"),
                cmd: other.to_string(),
                params: None,
            },
        }
    }
}

type SharedRobots = Arc<AsyncMutex<Vec<Robot>>>;

#[derive(Clone)]
pub struct Neato {
    mqtt_client: MqttClient,
    settings: NeatoSettings,
    robots: SharedRobots,
    last_state_update: Arc<Mutex<Option<chrono::DateTime<chrono::Utc>>>>,
}

impl Neato {
    pub fn new(mqtt_client: MqttClient, neato_settings: &NeatoSettings) -> Neato {
        Neato {
            mqtt_client,
            settings: neato_settings.clone(),
            robots: Arc::new(AsyncMutex::new(Vec::new())),
            last_state_update: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn init(mut self) -> color_eyre::Result<Neato> {
        info!("Initializing Neato cloud integration");
        self.robots = Arc::new(AsyncMutex::new(get_robots(&self.settings.clone()).await?));
        // let robots_with_states = update_robot_states(get_robots(neato_settings).await?).await?;

        for robot in self.robots.lock().await.iter() {
            let _r = robot.clone();
            info!("Found robot: {:?}", robot.name);
            debug!("Robot info: {:?}", robot);
        }

        // Start the state polling loop
        match self.init_polling().await {
            Ok(_) => (),
            Err(err) => {
                error!("Error initializing polling: {}", err);
            }
        }

        match self.init_react_to_subscription_messages().await {
            Ok(_) => (),
            Err(err) => {
                error!("Error initializing subscription messages: {}", err);
            }
        };

        info!("Neato connection initialized");

        Ok(self)
    }

    async fn force_update_states(&self) -> color_eyre::Result<()> {
        for robot in self.robots.lock().await.iter_mut() {
            debug!("Robot info before update: {:?}", robot);

            let result = send_command(robot, &RobotCmd::GetRobotState).await?;
            let serialized_result: NeatoState = serde_json::from_str(&result).unwrap();
            robot.state = Some(serialized_result);
            // robot.state = robot_map.get(&robot.serial).unwrap().state.clone();
            debug!("Robot info after update: {:?}\n", robot);
        }

        *self.last_state_update.lock().unwrap() = Some(Utc::now());

        Ok(())
    }

    async fn update_states(&self) -> color_eyre::Result<()> {
        // don't update states if we've done it recently
        // let last_update = self.last_state_update.lock().unwrap().clone();
        match self.last_state_update.try_lock() {
            Ok(last_state_update) => {
                if last_state_update.is_some()
                    && (Utc::now() - last_state_update.unwrap()).num_seconds()
                        < self.settings.cache_timeout as i64
                {
                    let ms_ago = (Utc::now() - last_state_update.unwrap()).num_milliseconds();
                    info!(
                        "Skipping update, last update was only {:.2} seconds ago.",
                        ms_ago as f64 / 1000.0
                    );
                    return Ok(());
                } else {
                    debug!("Updating robot states");
                }
            }
            Err(_) => {
                info!("Skipping update, something else seems to be updating the state");
                return Ok(());
            }
        }

        // Now we lock the robots again and update the state of each robot
        self.force_update_states().await?;

        Ok(())
    }

    async fn init_polling(&self) -> color_eyre::Result<()> {
        let poll_rate = Duration::from_millis(self.settings.poll_interval as u64 * 1000);
        let neato = self.clone();
        let mqtt_client = self.mqtt_client.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(poll_rate);

            loop {
                interval.tick().await;
                match neato.update_states().await {
                    Ok(_) => (),
                    Err(err) => {
                        info!("Error updating robot states: {}", err);
                    }
                };

                for robot in neato.robots.lock().await.iter() {
                    robot.publish(mqtt_client.clone()).await.unwrap();
                }
            }
        });
        Ok(())
    }

    async fn init_react_to_subscription_messages(&self) -> color_eyre::Result<()> {

        let mut s = self.clone();

        tokio::spawn(async move {
            loop {
                s.mqtt_client
                    .rx
                    .changed()
                    .await
                    .expect("Expected rx channel never to close");
                let msg = s.mqtt_client.rx.borrow().clone();
    
                println!("Received update instruction! Device: {:?}", msg);
    
                match msg {
                    Some(SendAction { action: RobotCmd::GetRobotState, .. }) => {
                        debug!("We don't do state updates from set messages");
                    }
                    Some(SendAction { action, id }) => {
                        let robot = s.robots
                            .lock().await
                            .clone()
                            .into_iter()
                            .find(|r| r.name == id)
                            .unwrap();
                        // let r = robot.clone().unwrap();
                        // info!("{}: Starting cleaning", id);
                        info!("{}: {}", robot.name, action);
                        if s.settings.dry_run {
                            info!("Dry run enabled, not sending command");
                        } else {
                            send_command(&robot, &action).await.unwrap();
                        }
                    }
                    _ => {}
                };
            }
        });

        Ok(())
    }
}

async fn get_robots(config: &NeatoSettings) -> Result<Vec<Robot>> {
    let body = AuthBody {
        email: config.email.clone(),
        password: config.password.clone(),
    };

    let token = surf::post(&format!("{}/sessions", BASE_URL))
        .body(surf::Body::from_json(&body).map_err(|err| eyre!(err))?)
        .await
        .map_err(|err| eyre!(err))?
        .body_json::<SessionsResponse>()
        .await
        .map_err(|err| eyre!(err))?
        .access_token;

    let robots = surf::get(&format!("{}/users/me/robots", BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|err| eyre!(err))?
        // .body_string() // in case you want to debug the whole response
        .body_json::<Vec<Robot>>()
        .await
        .map_err(|err| eyre!(err))?;

    Ok(robots)
}

async fn send_command(robot: &Robot, cmd: &RobotCmd) -> Result<String> {
    // https://developers.neatorobotics.com/api/nucleo
    let robot_message = cmd.build_robot_message();

    debug!(
        "Robot name {}, Sending command: {:?}",
        robot.name, robot_message
    );

    let body = serde_json::to_string(&robot_message)?;
    let serial = robot.serial.to_lowercase();
    let date: String = format!("{}", Utc::now().format("%a, %d %b %Y %H:%M:%S GMT"));
    let string_to_sign = format!("{}\n{}\n{}", serial, date, body);

    // Create HMAC-SHA256 instance which implements `Mac` trait
    let mut mac = HmacSha256::new_from_slice(robot.secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(string_to_sign.as_bytes());

    let signature = hex::encode(mac.finalize().into_bytes());

    let result = surf::post(&format!(
        "{}/vendors/neato/robots/{}/messages",
        robot.nucleo_url, robot.serial
    ))
    .header("Accept", "application/vnd.neato.nucleo.v1")
    .header("Date", date)
    .header("Authorization", format!("NEATOAPP {}", signature))
    .body(surf::Body::from_json(&robot_message).map_err(|err| eyre!(err))?)
    .await
    .map_err(|err| eyre!(err))?
    .body_string()
    .await
    .map_err(|err| eyre!(err))?;

    debug!("response: {}", result);

    Ok(result)
}
