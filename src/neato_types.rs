use std::fmt;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Robot {
    pub mac_address: String,
    pub model: String,
    pub name: String,
    pub nucleo_url: String,
    pub secret_key: String,
    pub serial: String,
    pub state: Option<NeatoState>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct PublicRobot {
    pub mac_address: String,
    pub model: String,
    pub name: String,
    pub nucleo_url: String,
    pub serial: String,
    pub state: Option<NeatoState>,
}

#[derive(Serialize, Debug)]
pub struct HouseCleaningParams {
    /// Should be set to 4 for persistent map
    /// 2 without persistent map
    pub category: u32,

    /// 1 is eco, 2 is turbo
    pub mode: u32,

    /// 1 is normal, 2 is extra care, 3 is deep. 3 requires mode = 2.
    #[serde(rename = "navigationMode")]
    pub navigation_mode: u32,
}

#[derive(Serialize, Debug)]
pub struct RobotMessage {
    #[serde(rename = "reqId")]
    pub req_id: String,
    pub cmd: String,
    pub params: Option<HouseCleaningParams>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct RobotStateDetails {
    #[serde(rename = "isCharging")]
    pub is_charging: bool,
    #[serde(rename = "isDocked")]
    pub is_docked: bool,
    #[serde(rename = "isScheduleEnabled")]
    pub is_schedule_enabled: bool,
    #[serde(rename = "dockHasBeenSeen")]
    pub dock_has_been_seen: bool,
    pub charge: i8,
}

#[derive(Clone, Copy, Serialize_repr, Deserialize_repr, Debug, PartialEq)]
#[repr(u8)]
pub enum RobotState {
    Invalid = 0,
    Idle = 1,
    Busy = 2,
    Paused = 3,
    Error = 4,
}

impl fmt::Display for RobotState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

// https://developers.neatorobotics.com/api/robot-remote-protocol/request-response-formats#13-6-strong-code-action-code-em-integer-em-strong
// action: integer
// If the state is busy, this element specifies what the robot is or has been busy doing.
// If the state is pause or error, it specifies the activity that the Robot was doing.
// If state is other, this element is null.
#[derive(Clone, Copy, Serialize_repr, Deserialize_repr, Debug)]
#[repr(u8)]
pub enum RobotAction {
    Invalid = 0,
    HouseCleaning = 1,
    SpotCleaning = 2,
    ManualCleaning = 3,
    Docking = 4,
    UserMenuActive = 5,
    SuspendedCleaning = 6,
    Updating = 7,
    CopyingLogs = 8,
    RecoveringLocation = 9,
    IECTest = 10,
    MapCleaning = 11,
    ExploringMap = 12,
    AcquiringPersisntentMapIDs = 13,
    CreatingAndUploadingMap = 14,
    SuspendedExploration = 15,
}

impl fmt::Display for RobotAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct NeatoState {
    pub alert: Option<String>,
    pub error: Option<String>,
    pub details: RobotStateDetails,
    pub state: RobotState,
    pub action: RobotAction,
}
