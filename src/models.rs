use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub input: String,
    pub status: Status,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskRequest {
    pub input: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: String,
    pub input: String,
    pub status: Status,
}

impl TaskResponse {
    pub fn from_id_and_task(id: &String, task: &Task) -> Self {
        Self {
            id: id.to_string(),
            input: task.input.clone(),
            status: task.status.clone(),
        }
    }
}

pub fn hash(input: &String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.clone().into_bytes());
    format!("{:x}", hasher.finalize())
}

#[derive(Debug, Clone, Serialize, Deserialize, Ord, PartialEq, PartialOrd, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Queued,
    Processing,
    Done,
    Error,
}

pub type Que = Mutex<VecDeque<(String, String)>>;
pub type Database = Mutex<HashMap<String, Task>>;

pub struct ServiceState {
    pub que: Que,
    pub db: Database,
}
