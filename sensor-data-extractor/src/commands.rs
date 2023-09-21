use std::path::PathBuf;

use link_lib::Request;

pub enum Commands {
    Ping,
    SetSensorId,
    GetEverything,
    GetEverythingAndSave(PathBuf),
    ResetSensor,
    GetTime,
    SyncTime,
}

/*impl Commands {
    pub fn to_request(&self) -> Vec<Request> {}
}*/
