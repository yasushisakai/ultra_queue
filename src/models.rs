use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Mutex,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub type Que = Mutex<VecDeque<(String, String)>>;
pub type Database = Mutex<HashMap<String, Task>>;
pub type Votes = Mutex<HashMap<String, HashSet<String>>>;
pub type IdAlias = Mutex<HashMap<String, String>>;

#[derive(Default)]
pub struct ServiceState {
    pub que: Que,
    pub db: Database,
    pub votes: Votes,
    pub alias: IdAlias,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub input: String,
    pub status: Status,
    pub added: DateTime<Utc>,
    pub likes: u32,
}

impl Task {
    pub fn new(input: &String) -> Self {
        Self {
            input: input.clone(),
            status: Status::Queued,
            added: Utc::now(),
            likes: 0,
        }
    }
    pub fn increment(&mut self) {
        self.likes += 1;
    }

    pub fn decrement(&mut self) {
        self.likes -= 1;
    }
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
    pub likes: u32,
}

impl TaskResponse {
    pub fn from_id_and_task(id: &String, task: &Task) -> Self {
        Self {
            id: id.to_string(),
            input: task.input.clone(),
            status: task.status.clone(),
            likes: task.likes,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct VoteRequest {
    pub city_id: String,
    pub user_id: String,
    pub is_increment: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AliasRequest {
    pub user_id: String,
    pub alias: String,
}
